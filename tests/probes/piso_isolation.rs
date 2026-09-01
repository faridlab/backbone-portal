//! PISO — the MANDATORY customer-isolation probe class.
//!
//! Ported from this module's consuming-service predecessor (its
//! `tests/portal_isolation.rs`, preserved at tag v0.1.1) and retargeted
//! from the old selling/billing/support read models to the identity
//! module's OWN declared read models + detail verb. The two test
//! SHAPES survive verbatim:
//!
//! - **PISO-1** — seed two principals' rows; assert every read model
//!   returns exactly the caller's rows and nothing of the other's.
//! - **PISO-2** — the write verb stamps the SESSION identity (the
//!   verified principal), never anything the request carried; invalid
//!   input is refused.
//!
//! Proven-by-revert discipline (the class's heritage): delete the
//! `WHERE id = principal.user_id` arm from any read below — or the
//! identity stamp from the write verb — and these probes go RED.

use super::common::{seed_audit, seed_principal, TestDb};
use backbone_portal::application::service::portal_error::PortalError;
use backbone_portal::application::service::portal_surface::{
    PortalDetailPatch, PortalDocumentSurface,
};
use backbone_portal::application::service::token_service::{
    PortalPrincipal, DEFAULT_BEARER_TTL_HOURS,
};

/// The verified-principal fixture each read model keys on — the
/// identity comes from the (verified) session, never the request.
fn principal_of(user_id: uuid::Uuid, email: &str) -> PortalPrincipal {
    PortalPrincipal { user_id, email: email.to_string(), display_name: None }
}

/// PISO-1: every read model is ownership-scoped — principal A's views
/// carry exactly A's record and A's access history; B's carry B's; no
/// cross-leak in either direction.
#[tokio::test]
async fn piso1_reads_are_principal_scoped() {
    let db = TestDb::new("piso1").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());

        // Two principals, each with their own record + own audit trail.
        let a = seed_principal(&db.pool, "a@example.com", "active").await;
        let b = seed_principal(&db.pool, "b@example.com", "active").await;
        // Distinct detail payloads so a mis-scoped read is visible.
        sqlx::query("UPDATE portal.portal_users SET phone = '111', city = 'A-city' WHERE id = $1")
            .bind(a)
            .execute(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("seed A details: {e}"));
        sqlx::query("UPDATE portal.portal_users SET phone = '222', city = 'B-city' WHERE id = $1")
            .bind(b)
            .execute(&db.pool)
            .await
            .unwrap_or_else(|e| panic!("seed B details: {e}"));
        // A's history has two events, B's has one — a leaked row changes
        // the counts.
        seed_audit(&db.pool, a, "login_succeeded").await;
        seed_audit(&db.pool, a, "bearer_minted").await;
        seed_audit(&db.pool, b, "login_succeeded").await;

        let a_view = svc
            .surface
            .my_details(&principal_of(a, "a@example.com"))
            .await
            .unwrap_or_else(|e| panic!("A my_details: {e}"));
        assert_eq!(a_view.user_id, a, "A's view is A's row");
        assert_eq!(a_view.phone.as_deref(), Some("111"), "A's view carries A's phone");
        assert_eq!(a_view.city.as_deref(), Some("A-city"));

        let b_view = svc
            .surface
            .my_details(&principal_of(b, "b@example.com"))
            .await
            .unwrap_or_else(|e| panic!("B my_details: {e}"));
        assert_eq!(b_view.user_id, b);
        assert_eq!(b_view.phone.as_deref(), Some("222"));
        assert_eq!(b_view.city.as_deref(), Some("B-city"));

        let a_history = svc
            .surface
            .my_access_history(&principal_of(a, "a@example.com"), 100)
            .await
            .unwrap_or_else(|e| panic!("A history: {e}"));
        assert_eq!(a_history.len(), 2, "A sees exactly A's two events");

        let b_history = svc
            .surface
            .my_access_history(&principal_of(b, "b@example.com"), 100)
            .await
            .unwrap_or_else(|e| panic!("B history: {e}"));
        assert_eq!(b_history.len(), 1, "B sees exactly B's one event — nothing of A's leaks");

        // A revoked principal's reads refuse: access state gates every
        // read, not just login.
        let gone = seed_principal(&db.pool, "gone@example.com", "revoked").await;
        let refused = svc.surface.my_details(&principal_of(gone, "gone@example.com")).await;
        assert!(
            matches!(refused, Err(PortalError::NotFound(_))),
            "a revoked principal's read must refuse, got {refused:?}"
        );
    }
    db.dispose().await;
}

/// PISO-2: the write verb stamps the SESSION identity — the row records
/// the acting principal in `metadata.updated_by` — and invalid input is
/// refused with nothing written.
#[tokio::test]
async fn piso2_write_verb_stamps_session_identity() {
    let db = TestDb::new("piso2").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let a = seed_principal(&db.pool, "a@example.com", "active").await;
        let b = seed_principal(&db.pool, "b@example.com", "active").await;

        let patch = PortalDetailPatch {
            phone: Some("999".into()),
            city: Some("A-new-city".into()),
            ..Default::default()
        };
        let view = svc
            .surface
            .update_my_details(&principal_of(a, "a@example.com"), patch)
            .await
            .unwrap_or_else(|e| panic!("A update: {e}"));
        assert_eq!(view.phone.as_deref(), Some("999"));

        // The identity stamp: the write recorded WHO wrote it — the
        // acting principal, always (verified by direct SQL, the way the
        // predecessor class verified its ticket row).
        let stamp: Option<String> = sqlx::query_scalar(
            r#"SELECT metadata->>'updated_by' FROM portal.portal_users WHERE id = $1"#,
        )
        .bind(a)
        .fetch_one(&db.pool)
        .await
        .unwrap_or_else(|e| panic!("read A stamp: {e}"));
        assert_eq!(
            stamp.as_deref(),
            Some(a.to_string().as_str()),
            "the write must stamp the acting principal's id"
        );

        // B's row untouched by A's write.
        let b_phone: Option<String> =
            sqlx::query_scalar("SELECT phone FROM portal.portal_users WHERE id = $1")
                .bind(b)
                .fetch_one(&db.pool)
                .await
                .unwrap_or_else(|e| panic!("read B phone: {e}"));
        assert_eq!(b_phone, None, "A's write must not touch B's row");

        // Invalid input refused, nothing written: the empty patch ...
        let empty = svc
            .surface
            .update_my_details(&principal_of(a, "a@example.com"), PortalDetailPatch::default())
            .await;
        assert!(
            matches!(empty, Err(PortalError::InvalidInput(_))),
            "empty patch must refuse, got {empty:?}"
        );
        // ... and the over-length value.
        let too_long = PortalDetailPatch {
            city: Some("x".repeat(81)),
            ..Default::default()
        };
        let refused = svc
            .surface
            .update_my_details(&principal_of(a, "a@example.com"), too_long)
            .await;
        assert!(
            matches!(refused, Err(PortalError::InvalidInput(_))),
            "over-length city must refuse, got {refused:?}"
        );
        let city: Option<String> =
            sqlx::query_scalar("SELECT city FROM portal.portal_users WHERE id = $1")
                .bind(a)
                .fetch_one(&db.pool)
                .await
                .unwrap_or_else(|e| panic!("read A city: {e}"));
        assert_eq!(city.as_deref(), Some("A-new-city"), "refused patches write nothing");
    }
    db.dispose().await;
}

/// PISO-3 (the class's bearer arm): a live minted bearer reads exactly
/// its OWN principal — the credential IS the identity; there is no
/// request-carried id anywhere on the read path.
#[tokio::test]
async fn piso3_bearer_resolves_exactly_its_principal() {
    let db = TestDb::new("piso3").await;
    {
        let svc = super::common::Svc::new(db.pool.clone());
        let a = seed_principal(&db.pool, "a@example.com", "active").await;
        let _b = seed_principal(&db.pool, "b@example.com", "active").await;

        let link = svc
            .tokens
            .mint_bearer(a, DEFAULT_BEARER_TTL_HOURS)
            .await
            .unwrap_or_else(|e| panic!("mint: {e}"));
        let verified = svc
            .tokens
            .verify_bearer(&link)
            .await
            .unwrap_or_else(|e| panic!("verify: {e}"));
        assert_eq!(verified.user_id, a, "the bearer resolves to exactly its own principal");
        assert_eq!(verified.email, "a@example.com");

        // The grant binds the principal: the MAC is recomputed over the
        // STORED row fields — a forged nonce/mac/exp on a valid id, or a
        // spliced id, refuses.
        let spliced = format!("{a}.deadbeefdeadbeefdeadbeefdeadbeef.9999999999.{}", "0".repeat(64));
        assert!(
            matches!(
                svc.tokens.verify_bearer(&spliced).await,
                Err(PortalError::CredentialRefused)
            ),
            "a forged link must refuse"
        );
    }
    db.dispose().await;
}
