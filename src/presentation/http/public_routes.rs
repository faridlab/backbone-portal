//! The portal's PUBLIC route surface (hand-written; user-owned; see
//! `metaphor.codegen.yaml`).
//!
//! The module DOES NOT SELF-MOUNT: it exports [`portal_public_routes`],
//! a plain `axum::Router` the composing host nests under the schema
//! name — `Router::new().nest("/api/v1/portal", portal_public_routes(..))`
//! (the route-naming ruling: schema names, not crate long-names). The
//! host decides exposure (which port, which gateway, whether at all);
//! the module only declares the shape.
//!
//! ## De-oracle at the edge
//!
//! Every credential refusal shares ONE body —
//! `{"error":"credential refused","code":"portal_credential_refused"}`:
//! unknown email, wrong password, revoked principal, unknown/expired/
//! rotated/forged bearer, unknown or spent invitation. No timing
//! oracle, no status oracle, no error-text oracle.
//!
//! ## Tier B at the edge
//!
//! Login/signup ride the escalating attempt book inside the service;
//! invitation redemption failures register on the same book here
//! (identity key + IP key) so all three hammerable verbs share one
//! throttle memory.
//!
//! ## Unknown JSON keys are dropped, not forwarded
//!
//! The request DTOs declare exactly the fields they read (serde's
//! default ignore-unknown): the PI-04 whitelist is enforced by the
//! [`crate::application::service::portal_surface::PortalDetailPatch`]
//! TYPE — an off-whitelist key in `PATCH /me` never reaches SQL.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::application::service::{
    access_service::AccessService,
    invite_service::InviteService,
    portal_error::PortalError,
    portal_surface::{PgPortalSurface, PortalDetailPatch, PortalDocumentSurface},
    token_service::{AttemptBook, PortalPrincipal, TokenService, DEFAULT_BEARER_TTL_HOURS},
};

/// The shared route state (all Arcs — cheap to clone into handlers).
#[derive(Clone)]
pub struct PortalPublicState {
    pub access: Arc<AccessService>,
    pub invites: Arc<InviteService>,
    pub tokens: Arc<TokenService>,
    pub surface: Arc<PgPortalSurface>,
    pub attempts: Arc<AttemptBook>,
}

impl PortalPublicState {
    /// Compose the whole public surface over one pool with explicit
    /// secrets (the host's one-call wiring: everything shares one
    /// throttle book, one token engine, one credential slot).
    pub fn compose(pool: sqlx::PgPool, bearer_secret: &[u8], recipient_secret: &[u8]) -> Self {
        let tokens = Arc::new(TokenService::with_secrets(pool.clone(), bearer_secret, recipient_secret));
        let invites = Arc::new(InviteService::new(pool.clone(), tokens.clone()));
        let slot = crate::application::service::credential_port::CredentialVerifierSlot::new();
        let access = Arc::new(AccessService::new(
            pool.clone(),
            tokens.clone(),
            crate::application::service::policy_service::PolicyService::new(pool.clone()),
            slot,
        ));
        let surface = Arc::new(PgPortalSurface::new(pool));
        let attempts = tokens.attempt_book();
        Self { access, invites, tokens, surface, attempts }
    }

    /// [`Self::compose`] reading both secrets from the environment (an
    /// unset secret stays empty and every mint/verify that needs it
    /// fails loudly with the typed secret-not-configured error — the
    /// fail-closed posture, never a zero-secret fallback).
    pub fn from_env(pool: sqlx::PgPool) -> Self {
        let bearer = std::env::var(crate::application::service::PORTAL_BEARER_SECRET_ENV)
            .unwrap_or_default();
        let recipient = std::env::var(crate::application::service::PORTAL_RECIPIENT_SECRET_ENV)
            .unwrap_or_default();
        Self::compose(pool, bearer.as_bytes(), recipient.as_bytes())
    }

    /// The credential slot the host installs its verifier into (sapiens
    /// owns the password hashes; unwired = loud refusal).
    pub fn credential_slot(&self) -> crate::application::service::credential_port::CredentialVerifierSlot {
        self.access.credential_slot()
    }
}

/// The uniform refusal body (the de-oracle constant — NEVER vary a
/// byte of it per failure arm).
fn refused() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({
            "error": "credential refused",
            "code": "portal_credential_refused",
        })),
    )
        .into_response()
}

/// Map a service error to the HTTP shape. The credential-shaped errors
/// collapse into the ONE uniform body here; everything else carries its
/// typed status + machine code.
fn portal_error_response(err: PortalError) -> Response {
    match err {
        PortalError::CredentialRefused | PortalError::InvalidCredentials => refused(),
        PortalError::RateLimited { retry_after_seconds } => (
            StatusCode::TOO_MANY_REQUESTS,
            [
                ("retry-after", retry_after_seconds.to_string()),
                ("content-type", "application/json".into()),
            ],
            Json(json!({
                "error": "rate limited",
                "code": "portal_rate_limited",
                "retry_after_seconds": retry_after_seconds,
            })),
        )
            .into_response(),
        other => {
            let status = StatusCode::from_u16(other.http_status() as u16)
                .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (
                status,
                Json(json!({
                    "error": other.to_string(),
                    "code": other.code(),
                })),
            )
                .into_response()
        }
    }
}

/// The caller IP for the throttle keys: the FIRST hop of
/// `X-Forwarded-For` (the client the gateway saw), falling back to
/// `"unknown"` — never trusts the header for authorization, only for
/// rate-shaping.
fn visitor_ip(headers: &HeaderMap) -> String {
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

/// The bearer principal behind `Authorization: Bearer <credential>` —
/// or the uniform refusal.
async fn bearer_principal(
    tokens: &TokenService,
    headers: &HeaderMap,
) -> Result<PortalPrincipal, Response> {
    let presented = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v: &HeaderValue| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    let Some(presented) = presented else {
        return Err(refused());
    };
    match tokens.verify_bearer(&presented).await {
        Ok(principal) => Ok(principal),
        Err(PortalError::CredentialRefused) => Err(refused()),
        Err(other) => Err(portal_error_response(other)),
    }
}

// ── request DTOs (the edge whitelist — unknown keys dropped) ──────────────────

#[derive(Debug, Deserialize)]
pub struct RedeemInviteRequest {
    pub link: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct PasswordRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RotateRequest {
    pub token: String,
}

// ── handlers ──────────────────────────────────────────────────────────────────

async fn redeem_invite(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
    Json(body): Json<RedeemInviteRequest>,
) -> Response {
    let ip = visitor_ip(&headers);
    match state.invites.redeem(&body.link, &body.email).await {
        Ok((_user_id, bearer)) => (
            StatusCode::OK,
            Json(json!({ "token": bearer, "token_type": "Bearer" })),
        )
            .into_response(),
        Err(PortalError::CredentialRefused) => {
            // Same throttle memory as login: the identity key the link
            // was presented for, plus the IP key.
            let key = format!("portal|id:{}", body.email.trim().to_lowercase());
            state.attempts.register_failure(&key, chrono::Utc::now());
            state
                .attempts
                .register_failure(&format!("portal|ip:{ip}"), chrono::Utc::now());
            refused()
        }
        Err(other) => portal_error_response(other),
    }
}

async fn login(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
    Json(body): Json<PasswordRequest>,
) -> Response {
    match state.access.login(&body.email, &body.password, &visitor_ip(&headers)).await {
        Ok((_user_id, bearer)) => (
            StatusCode::OK,
            Json(json!({ "token": bearer, "token_type": "Bearer" })),
        )
            .into_response(),
        Err(err) => portal_error_response(err),
    }
}

async fn signup(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
    Json(body): Json<PasswordRequest>,
) -> Response {
    match state.access.signup(&body.email, &body.password, &visitor_ip(&headers)).await {
        Ok((_user_id, bearer)) => (
            StatusCode::CREATED,
            Json(json!({ "token": bearer, "token_type": "Bearer" })),
        )
            .into_response(),
        Err(err) => portal_error_response(err),
    }
}

async fn rotate(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
    Json(body): Json<RotateRequest>,
) -> Response {
    // Throttle the rotate verb on the IP key (the presented credential
    // itself is the identity being rotated — failures still bite).
    let ip_key = format!("portal|ip:{}", visitor_ip(&headers));
    if let Err(err) = state.attempts.check(&ip_key, chrono::Utc::now()) {
        return portal_error_response(err);
    }
    match state.tokens.rotate_bearer(&body.token, DEFAULT_BEARER_TTL_HOURS).await {
        Ok(bearer) => (
            StatusCode::OK,
            Json(json!({ "token": bearer, "token_type": "Bearer" })),
        )
            .into_response(),
        Err(PortalError::CredentialRefused) => {
            state.attempts.register_failure(&ip_key, chrono::Utc::now());
            refused()
        }
        Err(other) => portal_error_response(other),
    }
}

async fn me(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
) -> Response {
    let Ok(principal) = bearer_principal(&state.tokens, &headers).await else {
        return refused();
    };
    match state.surface.my_details(&principal).await {
        Ok(view) => (StatusCode::OK, Json(serde_json::to_value(view).unwrap_or_default())).into_response(),
        Err(err) => portal_error_response(err),
    }
}

async fn update_me(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Response {
    let Ok(principal) = bearer_principal(&state.tokens, &headers).await else {
        return refused();
    };
    // Edge-side whitelist: forward ONLY the keys the patch type
    // declares; everything else in the body is dropped here.
    let Ok(patch) = serde_json::from_value::<PortalDetailPatch>(body) else {
        return portal_error_response(PortalError::InvalidInput(
            "the patch body did not match the writable detail fields".into(),
        ));
    };
    match state.surface.update_my_details(&principal, patch).await {
        Ok(view) => (StatusCode::OK, Json(serde_json::to_value(view).unwrap_or_default())).into_response(),
        Err(err) => portal_error_response(err),
    }
}

async fn my_access_history(
    State(state): State<PortalPublicState>,
    headers: HeaderMap,
) -> Response {
    let Ok(principal) = bearer_principal(&state.tokens, &headers).await else {
        return refused();
    };
    match state.surface.my_access_history(&principal, 50).await {
        Ok(events) => (
            StatusCode::OK,
            Json(json!({ "events": events
                .into_iter()
                .map(|e| json!({
                    "event": e.event,
                    "occurred_at": e.occurred_at.to_rfc3339(),
                    "actor": e.actor,
                }))
                .collect::<Vec<_>>() })),
        )
            .into_response(),
        Err(err) => portal_error_response(err),
    }
}

/// The portal's public router — the artifact the host nests under
/// `/api/v1/portal`. Pure export: this function mounts NOTHING.
pub fn portal_public_routes(state: PortalPublicState) -> Router {
    Router::new()
        .route("/auth/redeem-invite", post(redeem_invite))
        .route("/auth/login", post(login))
        .route("/auth/signup", post(signup))
        .route("/auth/rotate", post(rotate))
        .route("/me", get(me).patch(patch_me_adapter))
        .route("/me/access-history", get(my_access_history))
        .with_state(state)
}

/// `PATCH /me` shares the `update_me` handler (the method router needs
/// its own function token).
async fn patch_me_adapter(
    state: State<PortalPublicState>,
    headers: HeaderMap,
    body: Json<serde_json::Value>,
) -> Response {
    update_me(state, headers, body).await
}
