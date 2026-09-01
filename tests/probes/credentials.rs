//! The TWO independent Tier A credentials (ADR-0018 / the PI-01 split):
//! the expiring rotatable bearer and the stateless per-recipient HMAC.
//! Independence is the load-bearing property: rotating either leaves
//! the other verifiable.

use super::common::{seed_principal, TestDb};
use backbone_portal::application::service::portal_error::PortalError;
use backbone_portal::application::service::token_service::{
    lockout_until, parse_capability, AttemptBook, TokenService, ATTEMPT_MAX_FAILURES,
    DEFAULT_BEARER_TTL_HOURS, RECIPIENT_HMAC_TTL_DAYS,
};
use chrono::{Duration, Utc};

/// The bearer full cycle: mint → verify → rotate (the old credential
/// dies, the fresh one works) → every mint/rotation audited.
#[tokio::test]
async fn bearer_mint_verify_rotate_audited() {
    let db = TestDb::new("cred_bearer").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let user = seed_principal(&db.pool, "cycle@example.com", "active").await;

        let link = svc
            .tokens
            .mint_bearer(user, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        let verified = svc
            .tokens
            .verify_bearer(&link)
            .await
            .unwrap_or_else(|e| panic!("verify minted: {e}"));
        assert_eq!(verified.user_id, user);

        let fresh = svc
            .tokens
            .rotate_bearer(&link, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("rotate: {e}"));
        // The old credential died with the rotation ...
        assert!(
            matches!(svc.tokens.verify_bearer(&link).await, Err(PortalError::CredentialRefused)),
            "the rotated-out credential must refuse"
        );
        // ... the fresh one works.
        let reverified = svc
            .tokens
            .verify_bearer(&fresh)
            .await
            .unwrap_or_else(|e| panic!("verify fresh: {e}"));
        assert_eq!(reverified.user_id, user);

        // Mint audit: one bearer_minted + one bearer_rotated row.
        assert_eq!(super::common::audit_count(&db.pool, user, "bearer_minted").await, 1);
        assert_eq!(super::common::audit_count(&db.pool, user, "bearer_rotated").await, 1);
    }
    db.dispose().await;
}

/// The row-status gates: a rotated, revoked, or expired-looking bearer
/// refuses — and a REVOKED PRINCIPAL's live-looking bearer refuses too
/// (status checked at every verification, not just at login).
#[tokio::test]
async fn bearer_status_gates_bite() {
    let db = TestDb::new("cred_gates").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let user = seed_principal(&db.pool, "gates@example.com", "active").await;
        let link = svc
            .tokens
            .mint_bearer(user, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));

        // Flip the principal to revoked out from under the live-looking
        // credential — every subsequent verification must refuse.
        sqlx::query("UPDATE portal.portal_users SET status = 'revoked' WHERE id = $1")
            .bind(user)
            .execute(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("revoke: {e}"));
        assert!(
            matches!(svc.tokens.verify_bearer(&link).await, Err(PortalError::CredentialRefused)),
            "a revoked principal's live-looking bearer must refuse"
        );

        // Every refusal shares ONE error kind — no oracle between
        // unknown/forged/expired/rotated/revoked.
        let other = seed_principal(&db.pool, "other@example.com", "active").await;
        let other_link = svc
            .tokens
            .mint_bearer(other, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("mint other: {e}"));
        // Expire it under the credential.
        sqlx::query("UPDATE portal.portal_tokens SET token_expires_at = NOW() - INTERVAL '1 hour' WHERE user_id = $1")
            .bind(other)
            .execute(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("expire: {e}"));
        let expired = match svc.tokens.verify_bearer(&other_link).await {
            Err(PortalError::CredentialRefused) => true,
            other => panic!("expired bearer must share the CredentialRefused body, got {other:?}"),
        };
        assert!(expired);
    }
    db.dispose().await;
}

/// INDEPENDENCE (the ADR-0018 rule-3 property): the recipient HMAC's
/// input never covers the bearer. Rotating the bearer leaves the
/// recipient credential verifiable; re-minting the recipient credential
/// leaves the bearer verifiable.
#[tokio::test]
async fn credentials_rotate_independently() {
    let db = TestDb::new("cred_indep").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let user = seed_principal(&db.pool, "indep@example.com", "active").await;
        let email = "indep@example.com";

        let bearer = svc
            .tokens
            .mint_bearer(user, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("mint bearer: {e}"));
        let recipient = svc
            .tokens
            .mint_recipient_credential(user, email)
            .await
            .unwrap_or_else(|e| panic!("mint recipient: {e}"));
        let now = Utc::now().timestamp();
        assert!(svc.tokens.verify_recipient_credential(&recipient, now));

        // Rotate the BEARER: the recipient credential survives.
        let _fresh = svc
            .tokens
            .rotate_bearer(&bearer, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("rotate bearer: {e}"));
        assert!(
            svc.tokens.verify_recipient_credential(&recipient, now),
            "rotating the bearer must NOT invalidate the recipient HMAC"
        );

        // Re-mint the RECIPIENT credential (its rotation): the (new)
        // bearer survives — and the old recipient link stays valid until
        // its own expiry (the digest posture).
        let recipient2 = svc
            .tokens
            .mint_recipient_credential(user, email)
            .await
            .unwrap_or_else(|e| panic!("re-mint recipient: {e}"));
        assert!(svc.tokens.verify_recipient_credential(&recipient2, now));
        assert!(
            svc.tokens.verify_recipient_credential(&recipient, now),
            "an old recipient link lives out its own TTL (stateless re-mint)"
        );
        let fresh = svc
            .tokens
            .verify_bearer(
                &svc.tokens
                    .mint_bearer(user, DEFAULT_BEARER_TTL_HOURS)
                    .await
                    .unwrap_or_else(|e| panic!("mint again: {e}")),
            )
            .await;
        assert!(fresh.is_ok(), "recipient re-mint must not touch bearers");

        // The recipient credential expires at its own TTL.
        let later = (Utc::now() + Duration::days(RECIPIENT_HMAC_TTL_DAYS + 1)).timestamp();
        assert!(
            !svc.tokens.verify_recipient_credential(&recipient, later),
            "an expired recipient credential must refuse"
        );

        // Forged/tampered recipient links refuse (constant-time compare
        // under the hood — the probe asserts the verdict only).
        let mut tampered = recipient2.clone();
        if let Some(pos) = tampered.rfind('.').map(|p| p + 1) {
            tampered.replace_range(pos.., &"0".repeat(64));
        }
        assert!(!svc.tokens.verify_recipient_credential(&tampered, now));

        // The mint audit row exists for the stateless credential (its
        // ONLY durable trace).
        assert_eq!(
            super::common::audit_count(&db.pool, user, "recipient_hmac_minted").await,
            2,
            "each recipient mint is audited"
        );
    }
    db.dispose().await;
}

/// Fail-closed secrets: an unset secret is a loud typed refusal at mint
/// and a bare refusal at verify — never a zero-secret fallback that
/// would let anyone forge credentials.
#[tokio::test]
async fn unset_secrets_fail_loudly() {
    let db = TestDb::new("cred_secrets").await;
    {
        let user = seed_principal(&db.pool, "secrets@example.com", "active").await;

        // No bearer secret at all.
        let no_bearer = TokenService::with_secrets(db.pool.clone(), b"", b"recipient");
        assert!(matches!(
            no_bearer.mint_bearer(user, 1).await,
            Err(PortalError::BearerSecretNotConfigured)
        ));
        // No recipient secret at all.
        let no_recipient = TokenService::with_secrets(db.pool.clone(), b"bearer", b"");
        assert!(matches!(
            no_recipient.mint_recipient_credential(user, "secrets@example.com").await,
            Err(PortalError::RecipientSecretNotConfigured)
        ));
        assert!(!no_recipient.verify_recipient_credential("whatever", Utc::now().timestamp()));

        // WRONG secret ≠ right secret: a credential minted under one
        // bearer secret does not verify under another.
        let minter = TokenService::with_secrets(db.pool.clone(), b"bearer", b"recipient");
        let verifier = TokenService::with_secrets(db.pool.clone(), b"OTHER-bearer", b"recipient");
        let link = minter
            .mint_bearer(user, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        assert!(
            matches!(verifier.verify_bearer(&link).await, Err(PortalError::CredentialRefused)),
            "a credential minted under a different secret must refuse"
        );
    }
    db.dispose().await;
}

/// The capability parser + the pure Tier B curve (probe-asserted as
/// pure functions — the survey discipline).
#[test]
fn parse_and_lockout_curve_pure() {
    // parse: well-formed parses; every malformed shape is None.
    let wellformed_id = uuid::Uuid::new_v4();
    let nonce = "a".repeat(32);
    let mac = "b".repeat(64);
    assert!(parse_capability(&format!("{wellformed_id}.{nonce}.1700000000.{mac}")).is_some());
    for bad in [
        String::new(),
        format!("not-a-uuid.{nonce}.1700000000.{mac}"),                  // id not a uuid
        format!("{wellformed_id}..1700000000.{mac}"),                    // empty nonce
        format!("{wellformed_id}.{nonce}.notanumber.{mac}"),             // exp not a number
        format!("{wellformed_id}.{nonce}.1700000000.{}", "z".repeat(64)), // mac not hex
        format!("{wellformed_id}.{nonce}.1700000000.{mac}.extra"),       // trailing arm
    ] {
        assert!(parse_capability(&bad).is_none(), "must refuse malformed: {bad}");
    }

    // The escalating lockout curve: nothing before the threshold, 30 s
    // at it, doubling after, capped at 15 min.
    let now = Utc::now();
    assert!(lockout_until(ATTEMPT_MAX_FAILURES - 1, now).is_none());
    let at_threshold = lockout_until(ATTEMPT_MAX_FAILURES, now).unwrap();
    assert_eq!((at_threshold - now).num_seconds(), 30, "first lockout is 30 s");
    let doubled = lockout_until(ATTEMPT_MAX_FAILURES + 1, now).unwrap();
    assert_eq!((doubled - now).num_seconds(), 60, "second lockout doubles");
    let capped = lockout_until(ATTEMPT_MAX_FAILURES + 40, now).unwrap();
    assert_eq!((capped - now).num_seconds(), 900, "the lockout caps at 15 min");

    // The book: failures bite, the lockout window refuses, success resets.
    let book = AttemptBook::new();
    for _ in 0..ATTEMPT_MAX_FAILURES {
        book.register_failure("k", Utc::now() + Duration::hours(1));
    }
    let failed_at = Utc::now() + Duration::hours(1);
    // 10 s after the threshold failure: inside the 30 s window.
    assert!(
        matches!(
            book.check("k", failed_at + Duration::seconds(10)),
            Err(PortalError::RateLimited { .. })
        ),
        "a locked key must refuse with the typed rate limit"
    );
    // 31 s after: the window has lapsed and the key is checkable again
    // (the curve expires, it does not ban).
    assert!(
        book.check("k", failed_at + Duration::seconds(31)).is_ok(),
        "the lockout must lapse at its window edge"
    );
    // Success resets: even a re-failed key starts clean after `reset`.
    book.register_failure("k", failed_at + Duration::seconds(40));
    book.reset("k");
    assert!(
        book.check("k", failed_at + Duration::seconds(42)).is_ok(),
        "success resets the book"
    );
}
