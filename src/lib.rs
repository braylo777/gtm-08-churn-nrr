//! Stage 8 — Predictive Churn & Net Revenue Retention (NRR).
//!
//! * `nrr`    — NRR / GRR bridge from two MRR snapshots: new, expansion, contraction, churn
//!   and reactivation per account, with the standard formulas.
//! * `health` — weighted health score from normalized signals, trend vs last period, risk
//!   bands, and a renewal-risk score that adds time to renewal and risk events.
//! * `alerts` — usage drop-off against each account's own baseline, and expansion signals
//!   (seat utilization, sustained usage growth).

mod api;
pub mod alerts;
pub mod health;
pub mod nrr;
pub mod util;

pub use api::dispatch;
use serde_json::Value;

pub const OPS: &[&str] = &["nrr_bridge", "health_scores", "usage_alerts"];

pub fn handle(op: &str, i: &Value) -> Result<Value, String> {
    match op {
        "nrr_bridge" => nrr::bridge_op(i),
        "health_scores" => health::health_op(i),
        "usage_alerts" => alerts::alerts_op(i),
        _ => Err(format!("unknown op `{op}`; known: {}", OPS.join(", "))),
    }
}
