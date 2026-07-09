// cucumber-rs step-definition stub.
// Steps are the ONLY place implementation detail lives — translate declarative
// Gherkin into real calls against the application/use-case layer (not the UI).
//
// Cargo.toml (dev-dependencies):
//   cucumber = "0.21"
//   tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
//
// Run via the workspace: `metaphor test` (never `cargo test` from the workspace root).

use cucumber::{given, then, when, World};

// Holds state across the steps of one scenario.
#[derive(Debug, Default, World)]
pub struct FlowWorld {
    // e.g. the aggregate under test, the captured result/error
    // cart: Cart,
    // outcome: Option<Result<OrderConfirmed, OrderRejected>>,
}

#[given(regex = r"^<precondition phrasing>$")]
async fn precondition(_w: &mut FlowWorld) {
    // arrange state; no assertions here
}

#[when(regex = r"^<the single trigger>$")]
async fn trigger(_w: &mut FlowWorld) {
    // invoke the use case; capture the outcome on the World
}

#[then(regex = r"^<observable postcondition>$")]
async fn postcondition(_w: &mut FlowWorld) {
    // assert on the captured outcome — business-observable only
}

#[tokio::main]
async fn main() {
    FlowWorld::run("tests/features").await;
}
