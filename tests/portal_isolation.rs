//! Portal integration tests — the load-bearing invariant is **customer data isolation**: a portal user
//! sees only their own orders/invoices/tickets, and the write surface opens tickets only as themselves.
//! Composes the REAL selling/billing/support schemas + the REAL support write path.

use backbone_portal::PortalService;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

fn dburl() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5433/backbone_portal".into())
}
async fn pool() -> PgPool {
    PgPool::connect(&dburl()).await.expect("connect")
}

async fn seed_order(pool: &PgPool, company: Uuid, customer: Uuid, number: &str) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO selling.sales_orders
             (id, order_number, company_id, customer_id, status, order_date, currency, subtotal, tax_rate, tax_amount, total)
           VALUES ($1,$2,$3,$4,'draft'::sales_order_status, now()::date, 'IDR', 0, 0, 0, $5)"#,
    )
    .bind(id).bind(number).bind(company).bind(customer).bind(Decimal::new(150000, 0))
    .execute(pool).await.expect("seed order");
    id
}

async fn seed_invoice(pool: &PgPool, company: Uuid, customer: Uuid, number: &str, account: Uuid) -> Uuid {
    let id = Uuid::new_v4();
    sqlx::query(
        r#"INSERT INTO billing.sales_invoices
             (id, invoice_number, company_id, customer_id, status, posting_date, currency,
              net_total, tax_total, grand_total, outstanding_amount, receivable_account_id, posting_state)
           VALUES ($1,$2,$3,$4,'draft'::invoice_status, now()::date, 'IDR', 0, 0, $5, 0, $6, 'pending'::gl_posting_state)"#,
    )
    .bind(id).bind(number).bind(company).bind(customer).bind(Decimal::new(99000, 0)).bind(account)
    .execute(pool).await.expect("seed invoice");
    id
}

// PISO-1 — reads are scoped to the customer: customer A never sees B's orders/invoices/tickets.
#[tokio::test]
async fn piso1_reads_are_customer_scoped() {
    let pool = pool().await;
    let company = Uuid::new_v4();
    let a = Uuid::new_v4();
    let b = Uuid::new_v4();
    let account = Uuid::new_v4();
    let portal = PortalService::new(pool.clone());

    let a_order = seed_order(&pool, company, a, &format!("SO-A-{}", Uuid::new_v4())).await;
    seed_order(&pool, company, b, &format!("SO-B-{}", Uuid::new_v4())).await;
    let a_inv = seed_invoice(&pool, company, a, &format!("INV-A-{}", Uuid::new_v4()), account).await;
    seed_invoice(&pool, company, b, &format!("INV-B-{}", Uuid::new_v4()), account).await;
    let a_ticket = portal.submit_ticket(a, company, "A cannot log in".into(), None).await.unwrap();
    portal.submit_ticket(b, company, "B billing question".into(), None).await.unwrap();

    // A sees exactly its own.
    let orders = portal.my_orders(a).await.unwrap();
    assert_eq!(orders.len(), 1, "A sees only its order");
    assert_eq!(orders[0].id, a_order);
    let invoices = portal.my_invoices(a).await.unwrap();
    assert_eq!(invoices.len(), 1, "A sees only its invoice");
    assert_eq!(invoices[0].id, a_inv);
    let tickets = portal.my_tickets(a).await.unwrap();
    assert_eq!(tickets.len(), 1, "A sees only its ticket");
    assert_eq!(tickets[0].id, a_ticket);

    // Nothing of A's leaks into B's views.
    assert!(portal.my_orders(b).await.unwrap().iter().all(|o| o.id != a_order));
    assert!(portal.my_invoices(b).await.unwrap().iter().all(|i| i.id != a_inv));
    assert!(portal.my_tickets(b).await.unwrap().iter().all(|t| t.id != a_ticket));
}

// PISO-2 — the write surface stamps the session customer: a submitted ticket lands in support scoped to
// the submitting customer, and shows up in that customer's tickets only.
#[tokio::test]
async fn piso2_write_surface_stamps_customer() {
    let pool = pool().await;
    let company = Uuid::new_v4();
    let a = Uuid::new_v4();
    let portal = PortalService::new(pool.clone());

    let ticket = portal.submit_ticket(a, company, "Return request".into(), Some("Item arrived broken".into())).await.unwrap();

    // The REAL support issue exists, scoped to the customer.
    let (customer, status): (Option<Uuid>, String) = sqlx::query_as(
        "SELECT customer_id, status::text FROM support.issues WHERE id=$1")
        .bind(ticket).fetch_one(&pool).await.unwrap();
    assert_eq!(customer, Some(a));
    assert_eq!(status, "open");

    // A blank subject is refused.
    assert!(portal.submit_ticket(a, company, "  ".into(), None).await.is_err());
}
