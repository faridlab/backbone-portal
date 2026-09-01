//! Shared harness: one DISPOSABLE scratch database per test, FAIL-HARD.
//!
//! The suite never runs against a shared DB (and NEVER against any live
//! service database): each test mints `portal_seat_<marker>_<hex>` on
//! the local scratch Postgres (port 5433 — the pinned scratch
//! container), applies this module's migrations with a raw SQL file
//! runner, runs, and drops the database.
//!
//! FAIL-HARD CONTRACT (the survey/mailing harness class): a test that
//! cannot reach its scratch database PANICS — [`TestDb::new`] refuses
//! to return `None`, and [`skipped`] panics on principle. A skipped
//! probe is a FAILED probe: a green suite means the behaviors were
//! exercised, not that they were unreachable. Every failure branch
//! prints WHY before panicking.
//!
//! `TestDb::dispose()` is the explicit teardown; `Drop` is the leak
//! guard (best-effort DROP on a throwaway runtime) for tests that
//! panic.
//!
//! ## Proven-by-revert discipline (the isolation class's heritage)
//!
//! The customer-isolation probes in `piso_isolation.rs` descend from
//! this module's consuming-service predecessor (its `tests/
//! portal_isolation.rs`, preserved at tag v0.1.1): the honest way to
//! prove an ownership scope is to delete it, watch the probe go red,
//! restore it, and watch it go green again. Dropping the
//! `WHERE id = principal.user_id` arm from any read model, or the
//! identity stamp from the write verb, turns PISO-1/PISO-2 red.

use std::sync::Arc;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;

use backbone_portal::application::service::access_service::AccessService;
use backbone_portal::application::service::credential_port::{
    CredentialCheck, CredentialVerifierError, CredentialVerifierSlot, PortalCredentialVerifier,
};
use backbone_portal::application::service::invite_service::InviteService;
use backbone_portal::application::service::policy_service::PolicyService;
use backbone_portal::application::service::portal_surface::PgPortalSurface;
use backbone_portal::application::service::token_service::TokenService;

/// The scratch Postgres every test database is born on and dropped
/// from. Port 5433 is the pinned scratch container — NEVER a live
/// service database.
pub const SCRATCH_ADMIN_URL: &str = "postgres://postgres:postgres@localhost:5433/postgres";

/// The bearer-HMAC secret the probes share (explicit, never from the
/// environment — the probes must not depend on host configuration).
pub const PROBE_BEARER_SECRET: &[u8] = b"portal-probe-bearer-secret";

/// The per-recipient-HMAC secret (a DIFFERENT value than the bearer
/// secret — the independence the probes assert starts here).
pub const PROBE_RECIPIENT_SECRET: &[u8] = b"portal-probe-recipient-secret";

fn admin_url() -> String {
    std::env::var("PORTAL_TEST_ADMIN_URL").unwrap_or_else(|_| SCRATCH_ADMIN_URL.into())
}

/// The fail-hard skip: reaching this is a FAILURE, never a green tick.
pub fn skipped(reason: &str) -> ! {
    panic!("VACUOUS SKIP IS A FAILURE: {reason}");
}

/// One disposable scratch database, migrations applied. Panics (never
/// returns `None`) when the scratch Postgres is unreachable.
pub struct TestDb {
    pub pool: PgPool,
    name: String,
    admin: PgPool,
}

impl TestDb {
    pub async fn new(marker: &str) -> Self {
        let url = admin_url();
        let admin = match PgPoolOptions::new()
            .max_connections(4)
            .acquire_timeout(Duration::from_secs(5))
            .connect(&url)
            .await
        {
            Ok(a) => a,
            Err(e) => {
                eprintln!("PROBE-FAIL: {marker}: admin connect to {url} failed: {e}");
                skipped(&format!("scratch Postgres unreachable: {e}"));
            }
        };
        let suffix: String = Uuid::new_v4().simple().to_string().chars().take(8).collect();
        let name = format!("portal_seat_{marker}_{suffix}");
        // Disposable by construction: a stale DB of the same name goes first.
        if let Err(e) =
            sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{name}" WITH (FORCE)"#)).execute(&admin).await
        {
            eprintln!("PROBE-FAIL: {marker}: pre-drop of {name} failed: {e}");
            skipped(&format!("scratch pre-drop failed: {e}"));
        }
        if let Err(e) = sqlx::query(&format!(r#"CREATE DATABASE "{name}""#)).execute(&admin).await {
            eprintln!("PROBE-FAIL: {marker}: create database {name} failed: {e}");
            skipped(&format!("scratch create failed: {e}"));
        }
        // Splice ONLY the trailing path segment.
        let db_url = match url.rfind('/') {
            Some(i) => format!("{}{}", &url[..=i], name),
            None => url.clone(),
        };
        let pool = match PgPoolOptions::new()
            .max_connections(8)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&db_url)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                eprintln!("PROBE-FAIL: {marker}: connect to {db_url} failed: {e}");
                skipped(&format!("scratch connect failed: {e}"));
            }
        };
        if let Err(what) = apply_module_migrations(&pool, marker).await {
            skipped(&what);
        }
        Self { pool, name, admin }
    }

    /// Explicit teardown: drop the scratch database entirely.
    pub async fn dispose(self) {
        self.drop_db().await;
    }

    async fn drop_db(&self) {
        // FORCE: connected test pool may still hold an idle session.
        let _ = sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{}" WITH (FORCE)"#, self.name))
            .execute(&self.admin)
            .await;
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let name = self.name.clone();
        let url = admin_url();
        // Leak-guard teardown for panicking tests; dispose() is the happy path.
        std::thread::spawn(move || {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread().enable_all().build() {
                rt.block_on(async move {
                    if let Ok(admin) = sqlx::PgPool::connect(&url).await {
                        let _ = sqlx::query(&format!(
                            r#"DROP DATABASE IF EXISTS "{name}" WITH (FORCE)"#
                        ))
                        .execute(&admin)
                        .await;
                    }
                });
            }
        });
    }
}

/// Apply this module's migrations with a raw SQL file runner (sorted
/// `.up.sql` order — the module's files are self-contained).
async fn apply_module_migrations(pool: &PgPool, marker: &str) -> Result<(), String> {
    let manifest = env!("CARGO_MANIFEST_DIR");
    let dir = format!("{manifest}/migrations");
    let mut files: Vec<std::path::PathBuf> = match std::fs::read_dir(&dir) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.file_name().and_then(|n| n.to_str()).map(|n| n.ends_with(".up.sql")).unwrap_or(false)
            })
            .collect(),
        Err(e) => return Err(format!("PROBE-FAIL: {marker}: cannot read {dir}: {e}")),
    };
    files.sort();
    let mut conn = pool
        .acquire()
        .await
        .map_err(|e| format!("PROBE-FAIL: {marker}: cannot acquire pool conn: {e}"))?;
    for file in files {
        let sql = std::fs::read_to_string(&file)
            .map_err(|e| format!("PROBE-FAIL: {marker}: cannot read {}: {e}", file.display()))?;
        if let Err(e) = sqlx::raw_sql(&sql).execute(&mut *conn).await {
            return Err(format!("PROBE-FAIL: {marker}: migration {} failed: {e}", file.display()));
        }
    }
    Ok(())
}

// ── the service bundle (explicit secrets; no environment dependence) ─────────

/// Every hand service over one pool, sharing one throttle book and one
/// credential slot the probe can install doubles into.
pub struct Svc {
    pub tokens: Arc<TokenService>,
    pub invites: Arc<InviteService>,
    pub policy: PolicyService,
    pub access: Arc<AccessService>,
    pub surface: Arc<PgPortalSurface>,
    pub credential_slot: CredentialVerifierSlot,
}

impl Svc {
    pub fn new(pool: PgPool) -> Self {
        let tokens = Arc::new(TokenService::with_secrets(
            pool.clone(),
            PROBE_BEARER_SECRET,
            PROBE_RECIPIENT_SECRET,
        ));
        let invites = Arc::new(InviteService::new(pool.clone(), tokens.clone()));
        let policy = PolicyService::new(pool.clone());
        let credential_slot = CredentialVerifierSlot::new();
        let access = Arc::new(AccessService::new(
            pool.clone(),
            tokens.clone(),
            PolicyService::new(pool.clone()),
            credential_slot.clone(),
        ));
        let surface = Arc::new(PgPortalSurface::new(pool));
        Self { tokens, invites, policy, access, surface, credential_slot }
    }
}

// ── the credential-port fixture double ───────────────────────────────────────

/// A verifier double answering a fixed verdict chosen at construction
/// (the sapiens argon2 verifier's stand-in — the port contract, not
/// the hash, is what the probes exercise).
pub struct FakeVerifier {
    verdict: std::sync::atomic::AtomicBool,
}

impl FakeVerifier {
    pub fn agreeing() -> Arc<Self> {
        Arc::new(Self { verdict: std::sync::atomic::AtomicBool::new(true) })
    }

    pub fn refusing() -> Arc<Self> {
        Arc::new(Self { verdict: std::sync::atomic::AtomicBool::new(false) })
    }
}

#[async_trait::async_trait]
impl PortalCredentialVerifier for FakeVerifier {
    async fn verify(&self, _check: CredentialCheck<'_>) -> Result<bool, CredentialVerifierError> {
        Ok(self.verdict.load(std::sync::atomic::Ordering::SeqCst))
    }
}

// ── seeding helpers (direct SQL — tests may bypass the repositories) ────────

/// Insert a principal with the given status and return its id.
pub async fn seed_principal(pool: &PgPool, email: &str, status: &str) -> Uuid {
    let id = Uuid::new_v4();
    let res = sqlx::query(
        r#"INSERT INTO portal.portal_users (id, email, status)
           VALUES ($1, $2, $3::portal_user_status)"#,
    )
    .bind(id)
    .bind(email)
    .bind(status)
    .execute(pool)
    .await;
    if let Err(e) = res {
        panic!("PROBE-FAIL: seed_principal({email}, {status}): {e}");
    }
    id
}

/// Insert an audit row attributed to a principal (the read-model seed
/// for the isolation probes).
pub async fn seed_audit(pool: &PgPool, user: Uuid, event: &str) {
    let res = sqlx::query(
        r#"INSERT INTO portal.portal_audit_log (id, event, portal_user_id, actor)
           VALUES ($1, $2::portal_audit_event, $3, 'seed')"#,
    )
    .bind(Uuid::new_v4())
    .bind(event)
    .bind(user)
    .execute(pool)
    .await;
    if let Err(e) = res {
        panic!("PROBE-FAIL: seed_audit({user}, {event}): {e}");
    }
}

/// The count of audit rows for a principal with the given event.
pub async fn audit_count(pool: &PgPool, user: Uuid, event: &str) -> i64 {
    match sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*) FROM portal.portal_audit_log
           WHERE portal_user_id = $1 AND event = $2::portal_audit_event"#,
    )
    .bind(user)
    .bind(event)
    .fetch_one(pool)
    .await
    {
        Ok(n) => n,
        Err(e) => panic!("PROBE-FAIL: audit_count({user}, {event}): {e}"),
    }
}

// ── the lifecycle envelope fixture ───────────────────────────────────────────

/// A minimal but well-formed integration envelope for the subscription
/// probes (the wire shape the host's bus delivers).
pub fn sapiens_envelope(event_type: &str, payload: serde_json::Value) -> backbone_messaging::IntegrationEventEnvelope {
    backbone_messaging::IntegrationEventEnvelope {
        id: Uuid::new_v4().to_string(),
        event_type: event_type.to_string(),
        source_context: "sapiens".to_string(),
        aggregate_id: payload
            .get("user_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string(),
        occurred_at: chrono::Utc::now(),
        published_at: chrono::Utc::now(),
        version: 1,
        correlation_id: None,
        causation_id: None,
        payload,
    }
}
