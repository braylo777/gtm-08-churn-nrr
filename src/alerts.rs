//! Usage alerts. Baseline = mean of the weeks before the recent window. Drop-off fires when
//! every week in the recent window is below baseline by `drop_pct`. Expansion fires on high
//! seat utilization or growth above `growth_pct` sustained over the window.
use crate::util::*;
use serde_json::{json, Value};

pub fn alerts_op(i: &Value) -> R<Value> {
    let win = f_or(i, "recent_weeks", 3.0) as usize;
    let drop = f_or(i, "drop_pct", 30.0) / 100.0;
    let grow = f_or(i, "growth_pct", 25.0) / 100.0;
    let seat_hi = f_or(i, "seat_utilization_pct", 90.0) / 100.0;
    let mut out = vec![];
    for a in arr(i, "accounts")? {
        let id = s(a, "account")?;
        let w: Vec<f64> = arr(a, "weekly_usage")?.iter().map(|x| x.as_f64().ok_or_else(|| "weekly_usage must be numbers".to_string())).collect::<R<_>>()?;
        if w.len() < win + 2 {
            out.push(json!({"account": id, "alerts": [], "note": "not enough history"}));
            continue;
        }
        let (base_w, recent) = w.split_at(w.len() - win);
        let base = base_w.iter().sum::<f64>() / base_w.len() as f64;
        let rmean = recent.iter().sum::<f64>() / win as f64;
        let change = if base > 0.0 { rmean / base - 1.0 } else { 0.0 };
        let mut al = vec![];
        if base > 0.0 && recent.iter().all(|x| *x < base * (1.0 - drop)) {
            al.push(json!({"type": "usage_drop", "severity": if change < -0.6 { "high" } else { "medium" }, "detail": format!("last {win} weeks avg {:.0} vs baseline {:.0} ({:+.0}%)", rmean, base, change * 100.0)}));
        }
        if base > 0.0 && recent.iter().all(|x| *x > base * (1.0 + grow)) {
            al.push(json!({"type": "expansion_usage", "detail": format!("usage {:+.0}% vs baseline for {win} weeks", change * 100.0)}));
        }
        if let (Some(u), Some(l)) = (a.get("seats_used").and_then(Value::as_f64), a.get("seats_licensed").and_then(Value::as_f64)) {
            if l > 0.0 && u / l >= seat_hi {
                al.push(json!({"type": "expansion_seats", "detail": format!("{u}/{l} seats ({:.0}%)", u / l * 100.0)}));
            }
            if l > 0.0 && u / l < 0.5 {
                al.push(json!({"type": "shelfware", "severity": "medium", "detail": format!("only {u}/{l} seats used")}));
            }
        }
        out.push(json!({"account": id, "baseline": rd(base, 1), "recent_avg": rd(rmean, 1), "change_pct": rd(change * 100.0, 1), "alerts": al}));
    }
    let n = out.iter().filter(|x| !x["alerts"].as_array().unwrap().is_empty()).count();
    Ok(json!({"accounts_with_alerts": n, "accounts": out}))
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn drop_and_expansion() {
        let i = json!({"recent_weeks":2,"accounts":[{"account":"d","weekly_usage":[100,100,100,40,50]},
            {"account":"e","weekly_usage":[100,100,100,140,150],"seats_used":19,"seats_licensed":20},{"account":"n","weekly_usage":[1]}]});
        let r = alerts_op(&i).unwrap();
        assert_eq!(r["accounts"][0]["alerts"][0]["type"], "usage_drop");
        assert_eq!(r["accounts"][1]["alerts"].as_array().unwrap().len(), 2);
        assert_eq!(r["accounts_with_alerts"], 2);
    }
}
