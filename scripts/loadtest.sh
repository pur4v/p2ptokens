#!/usr/bin/env bash
# Load-test the coordinator's hot path (/match) and read path (/providers).
# Requires ApacheBench (`ab`). Builds release for a meaningful number.
set -uo pipefail
cd "$(dirname "$0")/.."

C=${C:-100}        # concurrency
N=${N:-300000}     # total requests
PORT=${PORT:-4000}

command -v ab >/dev/null || { echo "need ApacheBench (ab)"; exit 1; }

echo "=== building release coordinator ==="
cargo build --release -p p2ptokens-coordinator 2>&1 | tail -1

pkill -f p2p-coordinator 2>/dev/null || true; sleep 1
# --allow-unsigned-heartbeats: synthetic curl peers can't sign; INSECURE, load test only.
RUST_LOG=warn ./target/release/p2p-coordinator --listen 127.0.0.1:$PORT --allow-unsigned-heartbeats >/tmp/coord-load.log 2>&1 &
COORD=$!
trap 'kill $COORD 2>/dev/null || true' EXIT
sleep 1

# register one always-available provider to match against
curl -fsS http://127.0.0.1:$PORT/heartbeat -H 'content-type: application/json' \
  -d '{"peer_id":"prov1","multiaddrs":["/ip4/127.0.0.1/tcp/5000/p2p/prov1"],"offers":[{"model":{"name":"llama3.2:3b"},"backend":"ollama"}],"capacity":1000000,"in_flight":0}' >/dev/null
printf '%s' '{"consumer":"loadtester","model":{"name":"llama3.2:3b"}}' > /tmp/match.json

echo "=== cores: $(sysctl -n hw.ncpu 2>/dev/null || nproc)  |  concurrency=$C  requests=$N ==="
echo "--- POST /match (match + create job) ---"
ab -k -c "$C" -n "$N" -p /tmp/match.json -T application/json http://127.0.0.1:$PORT/match 2>&1 \
  | grep -E "Requests per second|Time per request|Non-2xx"
echo "--- GET /providers (read) ---"
ab -k -c "$C" -n "$N" http://127.0.0.1:$PORT/providers 2>&1 \
  | grep -E "Requests per second|Non-2xx"
echo "(note: ab 'Failed requests' counts length variance from the 1-in-5 audit flag; check Non-2xx for real errors)"
