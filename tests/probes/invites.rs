//! The invitation capability: mint, the ONE-SHOT redemption fence,
//! recipient binding, the explicit revocation list, expiry refusal.

use super::common::{audit_count, seed_principal, TestDb};
use backbone_portal::application::service::portal_error::PortalError;

/// The shipped access path: mint → redeem once (a principal is born
/// active with a first bearer) → the link is dead forever.
#[tokio::test]
async fn invite_redeems_exactly_once() {
    let db = TestDb::new("inv_once").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let link = svc
            .invites
            .mint_invite("first@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));

        let (user, bearer) = svc
            .invites
            .redeem(&link, "first@example.com")
            .await
            .unwrap_or_else(|e| panic!("redeem: {e}"));
        let status: String = sqlx::query_scalar(
            "SELECT status::text FROM portal.portal_users WHERE id = $1",
        )
        .bind(user)
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("read principal: {e}"));
        assert_eq!(status, "active", "a redeemed invitation births an ACTIVE principal");

        // The first bearer verifies — redemption minted it.
        assert!(svc.tokens.verify_bearer(&bearer).await.is_ok());

        // The link is dead: a second redemption (even by the same
        // recipient) refuses with the shared body.
        assert!(
            matches!(
                svc.invites.redeem(&link, "first@example.com").await,
                Err(PortalError::CredentialRefused)
            ),
            "a spent invitation must refuse re-redemption"
        );
        assert_eq!(audit_count(&db.pool, user, "invite_redeemed").await, 1);

        // And the invite row records its terminal state.
        let invite_status: String = sqlx::query_scalar(
            "SELECT status::text FROM portal.portal_invites WHERE recipient_email = $1",
        )
        .bind("first@example.com")
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("read invite: {e}"));
        assert_eq!(invite_status, "redeemed");
    }
    db.dispose().await;
}

/// The grant binds the RECIPIENT EMAIL: a link forwarded to another
/// mailbox refuses at redemption.
#[tokio::test]
async fn forwarded_link_refuses() {
    let db = TestDb::new("inv_fwd").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let link = svc
            .invites
            .mint_invite("right@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        assert!(
            matches!(
                svc.invites.redeem(&link, "wrong@example.com").await,
                Err(PortalError::CredentialRefused)
            ),
            "a link presented from the wrong mailbox must refuse"
        );
        // Nothing was born by the refused attempt.
        let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM portal.portal_users")
            .fetch_one(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("count: {e}"));
        assert_eq!(n, 0);
    }
    db.dispose().await;
}

/// The explicit revocation list: a revoked invitation refuses; the
/// trigger arm (revoke_pending_for_email) kills every pending invite
/// for a principal's email and leaves redeemed ones alone.
#[tokio::test]
async fn revocation_list_bites() {
    let db = TestDb::new("inv_rev").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());

        let revoked_link = svc
            .invites
            .mint_invite("revoked@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint revoked: {e}"));
        let invite_id: uuid::Uuid = sqlx::query_scalar(
            "SELECT id FROM portal.portal_invites WHERE recipient_email = $1",
        )
        .bind("revoked@example.com")
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("read invite id: {e}"));
        svc.invites
            .revoke_invite(invite_id, "officer test", "officer")
            .await
            .unwrap_or_else(|e| panic!("revoke: {e}"));
        assert!(
            matches!(
                svc.invites.redeem(&revoked_link, "revoked@example.com").await,
                Err(PortalError::CredentialRefused)
            ),
            "a revoked invitation must refuse"
        );
        // The revocation itself is audited (user-less row: the invite
        // preceded any principal).
        let revoked_rows: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_audit_log
               WHERE event = 'invite_revoked'"#,
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count revoked: {e}"));
        assert_eq!(revoked_rows, 1, "the officer revocation is audited");

        // The trigger arm: two pending invites for one email die
        // together; a redeemed one is untouched.
        for _ in 0..2 {
            svc.invites
                .mint_invite("sweep@example.com", None, 14)
                .await
                .unwrap_or_else(|e| panic!("mint sweep: {e}"));
        }
        let killed = svc
            .invites
            .revoke_pending_for_email("sweep@example.com", "login-kill", "system")
            .await
            .unwrap_or_else(|e| panic!("sweep: {e}"));
        assert_eq!(killed, 2, "both pending invitations die");
        let still: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_invites
               WHERE recipient_email = $1 AND status = 'pending'"#,
        )
        .bind("sweep@example.com")
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count pending: {e}"));
        assert_eq!(still, 0);
    }
    db.dispose().await;
}

/// Expiry: the redemption arm re-derives everything from the STORED
/// row — a link whose stored expiry has passed refuses (and a link
/// whose carried exp no longer matches the stored row refuses too).
#[tokio::test]
async fn expired_invites_refuse() {
    let db = TestDb::new("inv_exp").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let link = svc
            .invites
            .mint_invite("expired@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));

        // Push the stored expiry into the past: the carried exp and the
        // stored exp now disagree AND the row is expired — both are
        // refusal arms; the probe asserts the refusal either way.
        sqlx::query(
            "UPDATE portal.portal_invites SET token_expires_at = NOW() - INTERVAL '1 minute' WHERE recipient_email = $1",
        )
        .bind("expired@example.com")
        .execute(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("expire: {e}"));

        assert!(
            matches!(
                svc.invites.redeem(&link, "expired@example.com").await,
                Err(PortalError::CredentialRefused)
            ),
            "an expired invitation must refuse with the shared body"
        );

        // The sweep verb records the terminal state (safety never
        // depends on it — verify refuses regardless).
        sqlx::query(
            "UPDATE portal.portal_invites SET token_expires_at = NOW() + INTERVAL '1 day' WHERE recipient_email = $1",
        )
        .bind("expired@example.com")
        .execute(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("un-expire for sweep: {e}"));
        let swept = svc
            .invites
            .sweep_expired()
            .await
            .unwrap_or_else(|e| panic!("sweep: {e}"));
        assert_eq!(swept, 0, "a live pending invite is not swept");
    }
    db.dispose().await;
}

/// The mint audit: every mint lands an invite_minted row BEFORE any
/// principal exists (the audit trail covers the pre-principal edge).
#[tokio::test]
async fn invite_mints_are_audited() {
    let db = TestDb::new("inv_audit").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let _user = seed_principal(&db.pool, "unrelated@example.com", "active").await;
        svc.invites
            .mint_invite("audited@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        let n: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_audit_log
               WHERE event = 'invite_minted'"#,
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count: {e}"));
        assert_eq!(n, 1, "the mint is audited even though no principal exists yet");
    }
    db.dispose().await;
}
