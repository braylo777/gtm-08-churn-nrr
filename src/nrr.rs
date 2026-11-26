//! NRR = (start + expansion − contraction − churn) / start, from the starting cohort only.
//! GRR = (start − contraction − churn) / start (capped at 100%). New logos are outside NRR.
use crate::util::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;

pub fn bridge_op(i: &Value) -> R<Value> {
    let start = obj(i, "start_mrr")?;
    let end = obj(i, "end_mrr")?;
    let mut ids: Vec<&String> = start.keys().chain(end.keys()).collect();
    ids.sort();
    ids.dedup();
    let mut b: BTreeMap<&str, f64> = BTreeMap::new();
    let mut moves = vec![];
    for id in ids {
        let a = start.get(id).and_then(Value::as_f64).unwrap_or(0.0);
        let z = end.get(id).and_then(Value::as_f64).unwrap_or(0.0);
        if a < 0.0 || z < 0.0 {
            return Err(format!("{id}: MRR cannot be negative"));
        }
        let (kind, delta) = match (a > 0.0, z > 0.0) {
            (false, true) => {
                let react = i.get("previously_churned").and_then(Value::as_array).map_or(false, |v| v.iter().any(|x| x == id.as_str()));
                (if react { "reactivation" } else { "new" }, z)
            }
            (true, false) => ("churn", -a),
            (true, true) if z > a => ("expansion", z - a),
            (true, true) if z < a => ("contraction", z - a),
            (true, true) => ("flat", 0.0),
            (false, false) => continue,
        };
        *b.entry(kind).or_default() += delta.abs();
        if kind != "flat" {
            moves.push(json!({"account": id, "movement": kind, "from": a, "to": z, "delta": rd(delta, 2)}));
        }
    }
    let s0: f64 = start.values().filter_map(Value::as_f64).sum();
    if s0 <= 0.0 {
        return Err("starting MRR must be > 0".into());
    }
    let g = |k: &str| b.get(k).copied().unwrap_or(0.0);
    let nrr = (s0 + g("expansion") - g("contraction") - g("churn")) / s0;
    let grr = ((s0 - g("contraction") - g("churn")) / s0).min(1.0);
    let s1 = s0 + g("new") + g("reactivation") + g("expansion") - g("contraction") - g("churn");
    moves.sort_by(|x, y| x["delta"].as_f64().unwrap().total_cmp(&y["delta"].as_f64().unwrap()));
    Ok(json!({"start_mrr": rd(s0, 2), "end_mrr": rd(s1, 2),
        "bridge": {"new": g("new"), "reactivation": g("reactivation"), "expansion": g("expansion"), "contraction": -g("contraction"), "churn": -g("churn")},
        "nrr_pct": rd(nrr * 100.0, 1), "grr_pct": rd(grr * 100.0, 1), "movements": moves}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn nrr_math() {
        let i = json!({"start_mrr":{"a":100,"b":100,"c":100,"d":100},"end_mrr":{"a":150,"b":80,"d":100,"e":40,"f":10},"previously_churned":["f"]});
        let r = bridge_op(&i).unwrap();
        assert_eq!(r["nrr_pct"], 82.5); // (400+50-20-100)/400
        assert_eq!(r["grr_pct"], 70.0);
        assert_eq!(r["end_mrr"], 380.0);
        assert_eq!(r["bridge"]["reactivation"], 10.0);
    }
}
