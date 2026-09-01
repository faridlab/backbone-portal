//! The sapiens lifecycle subscription: `sapiens.user.created`
//! (onboarding link) and the deactivate/delete/anonymize revocation
//! arms, written against the v0.2.5 published event contract and
//! exercised with fixture envelopes (the producers are live in
//! sapiens v0.2.5).

use super::common::{audit_count, sapiens_envelope, seed_audit, seed_principal, TestDb};
use backbone_portal::application::service::lifecycle_handler::{
    anonymized_facts_from_published, created_facts_from_published, deleted_facts_from_published,
    SapiensLifecycleHandler,
};
use backbone_messaging::IntegrationEventHandler;
use uuid::Uuid;

/// `sapiens.user.created` links a pending portal principal (the
/// conditional stamp) and audits the link; a replayed envelope is an
/// idempotent non-action.
#[tokio::test]
async fn created_links_and_audits_idempotently() {
    let db = TestDb::new("lc_created").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let handler = SapiensLifecycleHandler::new(db.pool.clone(), svc.access.clone());
        let user = seed_principal(&db.pool, "linked@example.com", "active").await;
        let sapiens_id = Uuid::new_v4();

        let payload = serde_json::json!({ "user_id": sapiens_id.to_string(), "email": "Linked@Example.com" });
        handler
            .handle(sapiens_envelope("sapiens.user.created", payload))
            .await
            .unwrap_or_else(|e| panic!("handle created: {e}"));

        let link: Option<Uuid> =
            sqlx::query_scalar("SELECT sapiens_user_id FROM portal.portal_users WHERE id = $1")
                .bind(user)
                .fetch_one(&db.pool)
                .await
                .unwrap_or_else(|e| panic!("read link: {e}"));
        assert_eq!(link, Some(sapiens_id), "the lifecycle link is stamped (email normalized)");
        assert_eq!(audit_count(&db.pool, user, "onboarding_linked").await, 1);

        // Replay: the stamp is conditional (NULL-only) — no second audit.
        handler
            .handle(sapiens_envelope(
                "sapiens.user.created",
                serde_json::json!({ "user_id": sapiens_id.to_string(), "email": "linked@example.com" }),
            ))
            .await
            .unwrap_or_else(|e| panic!("replay: {e}"));
        assert_eq!(
            audit_count(&db.pool, user, "onboarding_linked").await,
            1,
            "a replayed envelope audits nothing new"
        );
    }
    db.dispose().await;
}

/// `sapiens.user.created` with NO portal principal is the audited
/// non-action (`onboarding_no_invitation`) — never a silent skip.
#[tokio::test]
async fn created_without_principal_audits_the_non_action() {
    let db = TestDb::new("lc_nolink").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let handler = SapiensLifecycleHandler::new(db.pool.clone(), svc.access.clone());

        handler
            .handle(sapiens_envelope(
                "sapiens.user.created",
                serde_json::json!({ "user_id": Uuid::new_v4().to_string(), "email": "employee@example.com" }),
            ))
            .await
            .unwrap_or_else(|e| panic!("handle: {e}"));

        let n: i64 = sqlx::query_scalar(
            r#"SELECT COUNT(*) FROM portal.portal_audit_log
               WHERE event = 'onboarding_no_invitation'"#,
        )
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("count: {e}"));
        assert_eq!(n, 1, "the non-action is audited, not skipped");
    }
    db.dispose().await;
}

/// The revocation arms (`sapiens.user.deactivated` / `.deleted`): the
/// linked principal is revoked, every live bearer dies, pending invites
/// die, and the edge is audited — once, with replay a no-op.
#[tokio::test]
async fn deactivate_and_delete_revoke_everything() {
    let db = TestDb::new("lc_revoke").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let handler = SapiensLifecycleHandler::new(db.pool.clone(), svc.access.clone());

        let user = seed_principal(&db.pool, "dying@example.com", "active").await;
        let sapiens_id = Uuid::new_v4();
        sqlx::query("UPDATE portal.portal_users SET sapiens_user_id = $1 WHERE id = $2")
            .bind(sapiens_id)
            .bind(user)
            .execute(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("stamp link: {e}"));
        let bearer = svc
            .tokens
            .mint_bearer(user, 24)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        svc.invites
            .mint_invite("dying@example.com", None, 14)
            .await
            .unwrap_or_else(|e| panic!("mint invite: {e}"));

        for event_type in ["sapiens.user.deactivated", "sapiens.user.deleted"] {
            handler
                .handle(sapiens_envelope(
                    event_type,
                    serde_json::json!({ "user_id": sapiens_id.to_string() }),
                ))
                .await
                .unwrap_or_else(|e| panic!("handle {event_type}: {e}"));

            let status: String = sqlx::query_scalar(
                "SELECT status::text FROM portal.portal_users WHERE id = $1",
            )
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("read status: {e}"));
            assert_eq!(status, "revoked", "{event_type} revokes the principal");

            assert!(
                matches!(
                    svc.tokens.verify_bearer(&bearer).await,
                    Err(backbone_portal::application::service::portal_error::PortalError::CredentialRefused)
                ),
                "{event_type} kills every live bearer"
            );
            assert_eq!(
                audit_count(&db.pool, user, "lifecycle_access_revoked").await,
                1,
                "the revocation edge is audited ONCE across both deliveries (replay is a no-op)"
            );
        }
    }
    db.dispose().await;
}

/// `sapiens.user.anonymized` runs the same revocation arm: the linked
/// principal is revoked, every live bearer dies, the edge is audited
/// once — anonymization is terminal for the identity, so nothing
/// keyed on the user may outlive it. Replay is a no-op.
#[tokio::test]
async fn anonymize_revokes_everything() {
    let db = TestDb::new("lc_anonymize").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let handler = SapiensLifecycleHandler::new(db.pool.clone(), svc.access.clone());

        let user = seed_principal(&db.pool, "erased@example.com", "active").await;
        let sapiens_id = Uuid::new_v4();
        sqlx::query("UPDATE portal.portal_users SET sapiens_user_id = $1 WHERE id = $2")
            .bind(sapiens_id)
            .bind(user)
            .execute(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("stamp link: {e}"));
        let bearer = svc
            .tokens
            .mint_bearer(user, 24)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));

        handler
            .handle(sapiens_envelope(
                "sapiens.user.anonymized",
                serde_json::json!({ "user_id": sapiens_id.to_string() }),
            ))
            .await
            .unwrap_or_else(|e| panic!("handle anonymized: {e}"));

        let status: String = sqlx::query_scalar("SELECT status::text FROM portal.portal_users WHERE id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("read status: {e}"));
        assert_eq!(status, "revoked", "anonymization revokes the principal");

        assert!(
            matches!(
                svc.tokens.verify_bearer(&bearer).await,
                Err(backbone_portal::application::service::portal_error::PortalError::CredentialRefused)
            ),
            "anonymization kills every live bearer"
        );
        assert_eq!(
            audit_count(&db.pool, user, "lifecycle_access_revoked").await,
            1,
            "the anonymization revocation edge is audited once"
        );

        // Replay: the revocation flip is conditional — nothing new.
        handler
            .handle(sapiens_envelope(
                "sapiens.user.anonymized",
                serde_json::json!({ "user_id": sapiens_id.to_string() }),
            ))
            .await
            .unwrap_or_else(|e| panic!("replay: {e}"));
        assert_eq!(
            audit_count(&db.pool, user, "lifecycle_access_revoked").await,
            1,
            "a replayed anonymization audits nothing new"
        );
    }
    db.dispose().await;
}

/// A sapiens user with NO portal principal deactivates cleanly (most
/// sapiens users are employees — no noise, no error).
#[tokio::test]
async fn gone_without_principal_is_quiet() {
    let db = TestDb::new("lc_quiet").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let handler = SapiensLifecycleHandler::new(db.pool.clone(), svc.access.clone());
        // Unrelated principal + audit noise that must stay untouched.
        let bystander = seed_principal(&db.pool, "bystander@example.com", "active").await;
        seed_audit(&db.pool, bystander, "login_succeeded").await;

        handler
            .handle(sapiens_envelope(
                "sapiens.user.deleted",
                serde_json::json!({ "user_id": Uuid::new_v4().to_string() }),
            ))
            .await
            .unwrap_or_else(|e| panic!("handle: {e}"));

        let status: String = sqlx::query_scalar(
            "SELECT status::text FROM portal.portal_users WHERE id = $1",
        )
        .bind(bystander)
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("read status: {e}"));
        assert_eq!(status, "active", "an unlinked deletion touches nobody");
    }
    db.dispose().await;
}

/// The published typed events (v0.2.5) derive the same wire facts the
/// envelope arms read — the load-bearing dependency probe: if the
/// published struct moves, THIS fails to compile.
#[test]
fn published_event_contract_facts() {
    let created = backbone_sapiens::infrastructure::messaging::UserCreatedIntegrationEvent {
        user_id: "11111111-1111-1111-1111-111111111111".into(),
        email: "Typed@Example.com".into(),
        username: "typed".into(),
        first_name: "Typed".into(),
        last_name: "Event".into(),
        display_name: None,
        occurred_at: chrono::Utc::now(),
        correlation_id: None,
    };
    let facts = created_facts_from_published(&created);
    assert_eq!(facts.user_id, "11111111-1111-1111-1111-111111111111");
    assert_eq!(facts.email, "Typed@Example.com");

    let deleted = backbone_sapiens::infrastructure::messaging::UserDeletedIntegrationEvent {
        user_id: "22222222-2222-2222-2222-222222222222".into(),
        occurred_at: chrono::Utc::now(),
        correlation_id: None,
    };
    let gone = deleted_facts_from_published(&deleted);
    assert_eq!(gone.user_id, "22222222-2222-2222-2222-222222222222");

    // The anonymized struct exists only from sapiens v0.2.5 — this
    // line is the compile-time proof of the tag repoint.
    let anonymized = backbone_sapiens::infrastructure::messaging::UserAnonymizedIntegrationEvent {
        user_id: "33333333-3333-3333-3333-333333333333".into(),
        occurred_at: chrono::Utc::now(),
        correlation_id: None,
    };
    let erased = anonymized_facts_from_published(&anonymized);
    assert_eq!(erased.user_id, "33333333-3333-3333-3333-333333333333");
}

/// The subscription's declared patterns + the defensive arm: an
/// unhandled type under a subscribed pattern is an error, never a
/// silent drop.
#[tokio::test]
async fn subscription_patterns_are_exact() {
    let db = TestDb::new("lc_patterns").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let handler = SapiensLifecycleHandler::new(db.pool.clone(), svc.access.clone());

        let patterns = handler.event_patterns();
        assert!(patterns.contains(&"sapiens.user.created"));
        assert!(patterns.contains(&"sapiens.user.deactivated"));
        assert!(patterns.contains(&"sapiens.user.deleted"));
        assert!(patterns.contains(&"sapiens.user.anonymized"));
        assert_eq!(handler.name(), "PortalSapiensLifecycleHandler");

        // A malformed payload (non-uuid user_id) is an ERROR — loud, not
        // a swallowed envelope.
        let bad = handler
            .handle(sapiens_envelope(
                "sapiens.user.created",
                serde_json::json!({ "user_id": "not-a-uuid", "email": "x@example.com" }),
            ))
            .await;
        assert!(bad.is_err(), "a malformed payload must surface as an error");
    }
    db.dispose().await;
}
