# gtm-08-churn-nrr

**GTM Engineering — Stage 8 of 12: Predictive Churn & Net Revenue Retention (NRR)**

Keep and grow the revenue you already have: an NRR / GRR bridge from MRR snapshots, explainable health and renewal-risk scores, and usage drop-off and expansion alerts against each account's own baseline.

**Live demo:** https://gtm-08-churn-nrr.braylo777.workers.dev — the Rust core runs in your browser as WebAssembly. No server, no tracking.

## Why this matters

For a SaaS company past early growth, net revenue retention decides the valuation more than new logos do. Churn rarely surprises the data: usage drops, the champion leaves, seats go unused, and renewal gets close. The work is to see it early, explain it in one line a CSM can act on, and catch the expansion signals on the same screen.

## What it does

- NRR / GRR bridge: new, reactivation, expansion, contraction, churn per account; formulas use the starting cohort only; GRR capped at 100%
- Health score: signals normalized between a bad and a good value (either direction), weights, trend vs last period, green / yellow / red bands, missing signals listed
- Renewal risk: low health + urgency from days to renewal + named risk events (champion left, P1 ticket), with the play to run and the biggest drag
- Usage alerts: drop-off when every recent week is below baseline by a threshold; expansion on sustained growth or seat utilization; shelfware under 50% seats

## What it does not do (yet)

- Risk weights are set by hand, not fit on historical churn (no logistic regression or survival model yet)
- No CRM / CS platform connection: snapshots and signals are synthetic JSON
- No seasonality adjustment in the usage baseline

## Operations

Every operation takes JSON in and gives JSON out. The CLI, the tests and the demo page all use the same entry point.

| Operation | What it does |
|---|---|
| `nrr_bridge` | Compare start-of-year and end-of-year MRR per account to get the NRR and GRR bridge. |
| `health_scores` | Score account health from five signals, then rank renewal risk with days to renewal and risk events. |
| `usage_alerts` | Compare the last 3 weeks with each account's own baseline. Flag drop-off, expansion and shelfware. |

## Run it

You need Rust (stable). No Node, no npm.

```bash
git clone https://github.com/braylo777/gtm-08-churn-nrr
cd gtm-08-churn-nrr
cargo test
cargo run -- ops
```

Try the first demo example from the command line:

```bash
python3 -c 'import json;print(json.dumps(json.load(open("web/examples.json"))["ops"][0]["input"]))' | cargo run -q -- nrr_bridge
```

## Build and deploy the demo

```bash
./build.sh    # tests, then compiles the core to web/app.wasm
./deploy.sh   # serves web/ as a Cloudflare Workers static-assets site (needs a Cloudflare login)
```

`web/` is a plain static site (`index.html`, `app.js`, `app.wasm`). It has no build step, so any static host works.

## Layout

```
src/lib.rs        operation list and router
src/api.rs        JSON entry point + the 3 WASM exports (alloc, dealloc, run)
src/main.rs       CLI
src/*.rs          one module per capability, each with unit tests
tests/            integration tests (every demo example must pass)
web/              demo page; examples.json drives both the page and this README
```

## Design choices

- **Rust, few dependencies.** Only `serde_json`, pinned to exact versions with a committed `Cargo.lock`.
- **Deterministic.** Time, randomness and keys are inputs, so every result can be replayed in a test or an audit.
- **Fail closed.** Unknown input or missing fields give an error, never a silent default.

## Part of a series

This is one of 12 repos that work through a public Go-To-Market (GTM) Engineering roadmap: [12 stages by Suraj Sharma (@suraj_sharma14)](https://x.com/suraj_sharma14/status/2125552583452270692). The roadmap is the inspiration; the code here is original. All 12: [github.com/braylo777?q=gtm-](https://github.com/braylo777?tab=repositories&q=gtm-).

## Data and privacy

Every example uses synthetic data. Nothing calls a paid API, scrapes a site, identifies a real visitor or sends an email.

## License

MIT — see [LICENSE](LICENSE). Built by [Braylo SI](https://braylo.si).
