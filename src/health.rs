//! Health score. Each signal is normalized to 0–100 between `bad` and `good` (either may be
//! the larger number), then weighted. Renewal risk adds urgency (days to renewal) and events
//! (champion left, open P1 ticket) on top of low health.
use crate::util::*;
use serde_json::{json, Value};

pub fn norm(x: f64, bad: f64, good: f64) -> f64 {
    if good == bad {
        return 50.0;
    }
    ((x - bad) / (good - bad)).clamp(0.0, 1.0) * 100.0
}

pub fn health_op(i: &Value) -> R<Value> {
    let model = arr(i, "model")?;
    let wsum: f64 = model.iter().map(|m| f_or(m, "weight", 0.0)).sum();
    if wsum <= 0.0 {
        return Err("model weights must add up to > 0".into());
    }
    let events = i.get("risk_events").and_then(Value::as_object).cloned().unwrap_or_default();
    let mut out = vec![];
    for a in arr(i, "accounts")? {
        let id = s(a, "account")?;
        let mut score = 0.0;
        let mut parts = vec![];
        let mut missing = vec![];
        for m in model {
            let sig = s(m, "signal")?;
            let w = f(m, "weight")? / wsum;
            match a.get("signals").and_then(|x| x.get(sig)).and_then(Value::as_f64) {
                None => {
                    missing.push(sig);
                    parts.push((sig, 0.0, w));
                }
                Some(v) => {
                    let n = norm(v, f(m, "bad")?, f(m, "good")?);
                    score += n * w;
                    parts.push((sig, n, w));
                }
            }
        }
        let prev = a.get("previous_score").and_then(Value::as_f64);
        let band = if score >= 70.0 { "green" } else if score >= 45.0 { "yellow" } else { "red" };
        let days = f_or(a, "days_to_renewal", 365.0);
        let urgency = (1.0 - days.clamp(0.0, 180.0) / 180.0) * 20.0;
        let mut ev_pts = 0.0;
        let mut ev_hit = vec![];
        for e in a.get("events").and_then(Value::as_array).into_iter().flatten().filter_map(Value::as_str) {
            let p = events.get(e).and_then(Value::as_f64).ok_or(format!("unknown risk event `{e}`"))?;
            ev_pts += p;
            ev_hit.push(e);
        }
        let risk = ((100.0 - score) * 0.6 + urgency + ev_pts).clamp(0.0, 100.0);
        let mut weakest: Vec<&(&str, f64, f64)> = parts.iter().collect();
        weakest.sort_by(|x, y| (x.1 * x.2 - 100.0 * x.2).total_cmp(&(y.1 * y.2 - 100.0 * y.2)));
        out.push(json!({"account": id, "health": rd(score, 1), "band": band, "trend": prev.map(|p| rd(score - p, 1)),
            "renewal_risk": rd(risk, 1), "risk_events": ev_hit, "days_to_renewal": days,
            "biggest_drag": weakest.first().map(|w| w.0), "missing_signals": missing,
            "play": if risk >= 60.0 { "exec save plan" } else if risk >= 40.0 { "CSM check-in this week" } else { "standard cadence" }}));
    }
    out.sort_by(|a, b| b["renewal_risk"].as_f64().unwrap().total_cmp(&a["renewal_risk"].as_f64().unwrap()));
    Ok(json!({"accounts": out}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn normalize_both_directions() {
        assert_eq!(norm(5.0, 0.0, 10.0), 50.0);
        assert_eq!(norm(2.0, 10.0, 0.0), 80.0); // fewer tickets is better
        assert_eq!(norm(99.0, 0.0, 10.0), 100.0);
    }
    #[test]
    fn risk_orders_accounts() {
        let i = json!({"model":[{"signal":"usage","weight":1,"bad":0,"good":100}],"risk_events":{"champion_left":25},
            "accounts":[{"account":"ok","signals":{"usage":90}},{"account":"bad","signals":{"usage":30},"days_to_renewal":30,"events":["champion_left"]}]});
        let r = health_op(&i).unwrap();
        assert_eq!(r["accounts"][0]["account"], "bad");
        assert_eq!(r["accounts"][0]["band"], "red");
        assert!(health_op(&json!({"model":[{"signal":"u","weight":1,"bad":0,"good":1}],"accounts":[{"account":"x","events":["nope"]}]})).is_err());
    }
}
