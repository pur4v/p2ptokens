#!/usr/bin/env bash
# End-to-end local smoke test:
#   coordinator + two unified clients (A seeds, B leeches from A via the swarm).
#
# Requires: a running Ollama with the model below pulled (`ollama pull llama3.2:3b`).
set -euo pipefail
cd "$(dirname "$0")/.."

MODEL="${MODEL:-llama3.2:3b}"
COORD="127.0.0.1:4000"
A_HTTP="127.0.0.1:8080"
B_HTTP="127.0.0.1:8081"
TMP="$(mktemp -d)"
PIDS=()

cleanup() { echo "--- shutting down ---"; for p in "${PIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT

echo "=== building ==="
cargo build -p p2ptokens-coordinator -p p2ptokens-client 2>&1 | tail -3

COORD_BIN=target/debug/p2p-coordinator
CLIENT_BIN=target/debug/p2ptokens

echo "=== starting coordinator ($COORD) ==="
"$COORD_BIN" --listen "$COORD" &
PIDS+=($!)
sleep 1

echo "=== starting client A (seeder, $A_HTTP) ==="
"$CLIENT_BIN" --coordinator "http://$COORD" --http "$A_HTTP" \
  --data-dir "$TMP/a" --p2p-listen /ip4/127.0.0.1/tcp/0 &
PIDS+=($!)

echo "=== starting client B (leecher, $B_HTTP) ==="
"$CLIENT_BIN" --coordinator "http://$COORD" --http "$B_HTTP" \
  --data-dir "$TMP/b" --p2p-listen /ip4/127.0.0.1/tcp/0 &
PIDS+=($!)

echo "=== waiting for heartbeats + listen addrs ==="
sleep 4

echo "=== B status (before) ==="
curl -fsS "http://$B_HTTP/api/status" | sed 's/,/,\n/g' | grep -E 'peer_id|swarm_size|consumed' | head

echo "=== leeching a completion from the swarm via client B ==="
curl -fsS "http://$B_HTTP/v1/chat/completions" \
  -H 'content-type: application/json' \
  -d "{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Say hello in exactly 5 words.\"}]}" \
  | python3 -m json.tool

echo "=== B status (after — expect consumed > 0) ==="
curl -fsS "http://$B_HTTP/api/status" | python3 -c 'import sys,json;d=json.load(sys.stdin);print("consumed:",d["consumed"],"ratio:",d["ratio"])'
echo "=== A status (after — expect served > 0) ==="
curl -fsS "http://$A_HTTP/api/status" | python3 -c 'import sys,json;d=json.load(sys.stdin);print("served:",d["served"])'

echo "=== done ==="
