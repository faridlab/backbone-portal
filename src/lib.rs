//! # backbone-portal
//!
//! Customer/supplier **self-service portal** — a **consuming service**, not a domain module. It owns no
//! schema and no bounded context; it composes read-models + a thin write surface over the PUBLIC contracts
//! of `backbone-selling`, `backbone-billing`, and `backbone-support` (tier5-deferred §5: "built as a
//! consuming service, not a new domain module, so it may never need its own bounded context").
//!
//! ## The load-bearing invariant: customer data isolation
//!
//! A portal user is ONE customer. **Every read is scoped to the authenticated `customer_id`** — a customer
//! must never see another customer's orders, invoices, or tickets. The scope is applied server-side (the
//! `customer_id` is the session identity, never a client-supplied filter the caller can widen). The write
//! surface stamps the same `customer_id` onto what it creates.

use rust_decimal::Decimal;
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub use backbone_support::application::service::support_write_service::{NewIssue, SupportError};

#[derive(Debug, thiserror::Error)]
pub enum PortalError {
    #[error("db: {0}")]
    Db(#[from] sqlx::Error),
    #[error("support: {0}")]
    Support(#[from] SupportError),
    #[error("invalid input: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct OrderView {
    pub id: Uuid,
    pub order_number: String,
    pub status: String,
    pub total: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InvoiceView {
    pub id: Uuid,
    pub invoice_number: String,
    pub status: String,
    pub total: Decimal,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TicketView {
    pub id: Uuid,
    pub subject: String,
    pub status: String,
}

/// The portal composition service. Constructed over the composed database; drives the modules' write
/// services for the write surface.
pub struct PortalService {
    pool: PgPool,
    support: backbone_support::application::service::support_write_service::SupportWriteService,
}

impl PortalService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            support: backbone_support::application::service::support_write_service::SupportWriteService::new(pool.clone()),
            pool,
        }
    }

    /// The customer's own sales orders — scoped to `customer_id`.
    pub async fn my_orders(&self, customer_id: Uuid) -> Result<Vec<OrderView>, PortalError> {
        let rows = sqlx::query(
            r#"SELECT id, order_number, status::text AS status, total
               FROM selling.sales_orders
               WHERE customer_id = $1 AND (metadata->>'deleted_at') IS NULL
               ORDER BY order_date DESC"#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| OrderView {
            id: r.get("id"), order_number: r.get("order_number"), status: r.get("status"), total: r.get("total"),
        }).collect())
    }

    /// The customer's own sales invoices — scoped to `customer_id`.
    pub async fn my_invoices(&self, customer_id: Uuid) -> Result<Vec<InvoiceView>, PortalError> {
        let rows = sqlx::query(
            r#"SELECT id, invoice_number, status::text AS status, grand_total AS total
               FROM billing.sales_invoices
               WHERE customer_id = $1 AND (metadata->>'deleted_at') IS NULL
               ORDER BY posting_date DESC"#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| InvoiceView {
            id: r.get("id"), invoice_number: r.get("invoice_number"), status: r.get("status"), total: r.get("total"),
        }).collect())
    }

    /// The customer's own support tickets — scoped to `customer_id`.
    pub async fn my_tickets(&self, customer_id: Uuid) -> Result<Vec<TicketView>, PortalError> {
        let rows = sqlx::query(
            r#"SELECT id, subject, status::text AS status
               FROM support.issues
               WHERE customer_id = $1 AND (metadata->>'deleted_at') IS NULL
               ORDER BY opened_at DESC"#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.iter().map(|r| TicketView {
            id: r.get("id"), subject: r.get("subject"), status: r.get("status"),
        }).collect())
    }

    /// The write surface: the customer submits a support ticket. The `customer_id` is stamped from the
    /// session — never taken from the request body — so a customer can only open tickets as themselves.
    pub async fn submit_ticket(
        &self,
        customer_id: Uuid,
        company_id: Uuid,
        subject: String,
        description: Option<String>,
    ) -> Result<Uuid, PortalError> {
        if subject.trim().is_empty() {
            return Err(PortalError::Invalid("a ticket needs a subject".into()));
        }
        let id = self.support.raise_issue(NewIssue {
            company_id,
            customer_id: Some(customer_id),
            subject,
            description,
            priority: "medium".into(),
            sla_id: None,
        }, chrono::Utc::now()).await?;
        Ok(id)
    }
}
