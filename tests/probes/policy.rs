//! The kill-switchable signup policy (PI-09/PI-18): DEFAULT OFF,
//! fail-closed, zero install-time bootstrap, flip audited; plus the
//! fail-closed credential port and the Tier B throttle at the access
//! verbs.

use super::common::{audit_count, seed_principal, FakeVerifier, TestDb};
use backbone_portal::application::service::portal_error::PortalError;

/// The shipped posture: NO policy row exists, signup reads CLOSED, the
/// signup verb refuses with the typed error and the refusal is audited.
#[tokio::test]
async fn signup_is_off_by_default_and_fail_closed() {
    let db = TestDb::new("pol_off").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        svc.credential_slot.install(FakeVerifier::agreeing());

        // Zero install-time bootstrap: nothing seeded the table.
        let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portal.portal_signup_policies")
            .fetch_one(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("count: {e}"));
        assert_eq!(rows, 0, "no policy row ships — bootstrap would be a defect");

        assert!(!svc.policy.signup_open().await, "absent row reads CLOSED");
        let refused = svc.access.signup("walkin@example.com", "pw", "1.2.3.4").await;
        assert!(
            matches!(refused, Err(PortalError::SignupClosed)),
            "signup against a closed policy must refuse with the typed error, got {refused:?}"
        );
        let n: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_audit_log
               WHERE event = 'signup_refused_policy_closed'"#,
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count refusals: {e}"));
        assert_eq!(n, 1, "the closed-policy refusal is audited");

        // Nothing was born.
        let principals: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portal.portal_users")
            .fetch_one(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("count principals: {e}"));
        assert_eq!(principals, 0);

        // An explicitly-disabled row reads CLOSED too (fail-closed on
        // every value but an explicit true).
        svc.policy
            .set_signup_policy(false, Some("probe"), None)
            .await
            .unwrap_or_else(|e| panic!("set false: {e}"));
        assert!(!svc.policy.signup_open().await);
    }
    db.dispose().await;
}

/// The kill switch actually switches: an officer flip opens signup, a
/// verified credential creates the principal (audited), and flipping
/// back OFF bites immediately — the in-flight attempt after the flip
/// refuses.
#[tokio::test]
async fn the_switch_flips_both_ways() {
    let db = TestDb::new("pol_flip").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let verifier = FakeVerifier::agreeing();
        svc.credential_slot.install(verifier);

        svc.policy
            .set_signup_policy(true, Some("probe open"), None)
            .await
            .unwrap_or_else(|e| panic!("open: {e}"));
        assert!(svc.policy.signup_open().await);

        let (user, bearer) = svc
            .access
            .signup("walkin@example.com", "pw", "1.2.3.4")
            .await
            .unwrap_or_else(|e| panic!("signup open: {e}"));
        assert_eq!(audit_count(&db.pool, user, "signup_created").await, 1);
        assert!(svc.tokens.verify_bearer(&bearer).await.is_ok());

        // A second signup of the SAME email behaves as a login (the
        // principal exists once — the DB fence).
        let (again, _) = svc
            .access
            .signup("walkin@example.com", "pw", "1.2.3.4")
            .await
            .unwrap_or_else(|e| panic!("signup again: {e}"));
        assert_eq!(again, user, "the same email stays ONE principal");
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portal.portal_users")
            .fetch_one(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("count: {e}"));
        assert_eq!(n, 1);

        // Kill it: the very next attempt refuses.
        svc.policy
            .set_signup_policy(false, Some("probe kill"), None)
            .await
            .unwrap_or_else(|e| panic!("kill: {e}"));
        assert!(
            matches!(
                svc.access.signup("late@example.com", "pw", "1.2.3.4").await,
                Err(PortalError::SignupClosed)
            ),
            "the kill switch bites immediately"
        );
        let flips: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_audit_log
               WHERE event = 'policy_changed'"#,
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count flips: {e}"));
        assert_eq!(flips, 2, "every flip is audited (open + kill)");
    }
    db.dispose().await;
}

/// The credential port is FAIL-CLOSED: unwired, a login for a perfectly
/// real active principal refuses LOUDLY (the typed not-composed error,
/// audited) — never a silent allow, never a fake 401.
#[tokio::test]
async fn unwired_credential_port_refuses_loudly() {
    let db = TestDb::new("pol_port").await;
    {
        let svc = super::common::Svc::new(db.pool.clone()); // NO verifier installed
        let user = seed_principal(&db.pool, "real@example.com", "active").await;
        assert!(!svc.credential_slot.is_wired());

        let refused = svc.access.login("real@example.com", "pw", "1.2.3.4").await;
        assert!(
            matches!(refused, Err(PortalError::CredentialPortNotComposed)),
            "an unwired verifier must refuse loudly, got {refused:?}"
        );
        assert_eq!(
            audit_count(&db.pool, user, "credential_port_not_composed").await,
            1,
            "the not-composed refusal is audited"
        );
    }
    db.dispose().await;
}

/// Login de-oracle + throttle: unknown email and wrong credential share
/// ONE refusal; hammering the verb trips Tier B (spacing or lockout —
/// either way the typed 429), per identity AND per IP.
#[tokio::test]
async fn login_is_de_oracled_and_throttled() {
    let db = TestDb::new("pol_thr").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let verifier = FakeVerifier::refusing();
        svc.credential_slot.install(verifier);
        seed_principal(&db.pool, "real@example.com", "active").await;

        // The uniform pair: unknown email and wrong credential are the
        // SAME error (no oracle). Each arm runs from its own IP so the
        // Tier B spacing gate cannot answer before the credential does.
        let unknown = svc.access.login("nobody@example.com", "pw", "9.9.9.1").await;
        let wrong = svc.access.login("real@example.com", "pw", "9.9.9.2").await;
        assert!(matches!(unknown, Err(PortalError::InvalidCredentials)));
        assert!(matches!(wrong, Err(PortalError::InvalidCredentials)));
        assert_eq!(
            format!("{unknown:?}").split('(').next(),
            format!("{wrong:?}").split('(').next(),
            "the two refusals are indistinguishable"
        );

        // A revoked principal refuses with the SAME body too (no status
        // oracle).
        seed_principal(&db.pool, "dead@example.com", "revoked").await;
        assert!(matches!(
            svc.access.login("dead@example.com", "pw", "9.9.9.3").await,
            Err(PortalError::InvalidCredentials)
        ));

        // Hammer: Tier B bites (the 1 s spacing gate fires on rapid
        // attempts — anti-hammering is Tier B by design).
        let mut tripped = false;
        for _ in 0..6 {
            if let Err(PortalError::RateLimited { .. }) =
                svc.access.login("real@example.com", "pw", "9.9.9.9").await
            {
                tripped = true;
                break;
            }
        }
        assert!(tripped, "a hammered login must hit the typed rate limit");

        // And the throttle is keyed per identity: a DIFFERENT identity
        // from the same IP is still checkable (its own counter is fresh;
        // the spacing gate is per key).
        tokio::time::sleep(std::time::Duration::from_millis(1100)).await;
        let other = svc.access.login("fresh@example.com", "pw", "9.9.9.9").await;
        assert!(
            !matches!(other, Err(PortalError::RateLimited { .. })),
            "a fresh identity starts with a fresh counter, got {other:?}"
        );
    }
    db.dispose().await;
}

/// Login success: the verifier's OK mints a bearer, resets the book,
/// touches last_login, and audits login_succeeded.
#[tokio::test]
async fn login_success_mints_and_audits() {
    let db = TestDb::new("pol_ok").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let verifier = FakeVerifier::agreeing();
        svc.credential_slot.install(verifier);
        let user = seed_principal(&db.pool, "ok@example.com", "active").await;

        let (uid, bearer) = svc
            .access
            .login("OK@example.com", "pw", "1.2.3.4")
            .await
            .unwrap_or_else(|e| panic!("login: {e}"));
        assert_eq!(uid, user, "the email normalizes to the same principal");
        assert!(svc.tokens.verify_bearer(&bearer).await.is_ok());
        assert_eq!(audit_count(&db.pool, user, "login_succeeded").await, 1);
        let touched: Option<chrono::DateTime<chrono::Utc>> =
            sqlx::query_scalar("SELECT last_login_at FROM portal.portal_users WHERE id = $1")
                .bind(user)
                .fetch_one(&db.pool)
                .await
                .unwrap_or_else(|e| panic!("read last_login: {e}"));
        assert!(touched.is_some(), "login touches last_login_at");
    }
    db.dispose().await;
}

/// The officer revocation sweep: revoke_access flips the principal,
/// kills every live bearer, and kills their pending invitations.
#[tokio::test]
async fn revoke_access_sweeps_everything() {
    let db = TestDb::new("pol_rev").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let user = seed_principal(&db.pool, "sweep@example.com", "active").await;
        let link = svc
            .tokens
            .mint_bearer(user, 24)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        svc.invites
            .mint_invite("sweep@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint invite: {e}"));

        let flipped = svc
            .access
            .revoke_access(user, "officer probe", "officer")
            .await
            .unwrap_or_else(|e| panic!("revoke: {e}"));
        assert!(flipped, "the first revocation performs the flip");
        assert!(
            matches!(svc.tokens.verify_bearer(&link).await, Err(PortalError::CredentialRefused)),
            "the revoked principal's bearer dies"
        );
        let status: String = sqlx::query_scalar(
            "SELECT status::text FROM portal.portal_users WHERE id = $1",
        )
        .bind(user)
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("read status: {e}"));
        assert_eq!(status, "revoked");
        let pending: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_invites
               WHERE recipient_email = 'sweep@example.com' AND status = 'pending'"#,
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count pending: {e}"));
        assert_eq!(pending, 0, "their pending invitations die with them");

        // Re-revocation flips nothing (the replay fence).
        let replay = svc
            .access
            .revoke_access(user, "replay", "officer")
            .await
            .unwrap_or_else(|e| panic!("re-revoke: {e}"));
        assert!(!replay, "a replayed revocation must be a no-op");
    }
    db.dispose().await;
}
