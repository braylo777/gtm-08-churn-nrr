#!/bin/sh
# Deploy web/ as a Cloudflare Workers static-assets site "gtm-08-churn-nrr" (no server code).
# Needs wrangler and a Cloudflare login. Set WRANGLER to override the pinned path.
set -eu
cd "$(dirname "$0")"
WRANGLER="${WRANGLER:-$HOME/bin/wrangler}"
./build.sh
"$WRANGLER" deploy
