#!/usr/bin/env bash
# Demonstrates the NAT-traversal topology:
#   coordinator + a public circuit-relay node + two clients that reserve a slot
#   on the relay (so NAT'd peers would be reachable), then a completion + settle.
set -uo pipefail
cd "$(dirname "$0")/.."

MODEL="${MODEL:-llama3.2:3b}"
COORD=127.0.0.1:4000
RELAY_P2P=/ip4/127.0.0.1/tcp/4001
TMP="$(mktemp -d)"
PIDS=()
cleanup(){ echo "--- shutdown ---"; for p in "${PIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT

C=target/debug/p2p-coordinator
P=target/debug/p2ptokens

echo "=== coordinator ==="
"$C" --listen $COORD >/tmp/coord.log 2>&1 & PIDS+=($!)
sleep 1

echo "=== relay node (public rendezvous, no backends) ==="
P2P_OLLAMA=0 RUST_LOG=info,libp2p=warn "$P" --relay \
  --p2p-listen $RELAY_P2P --http 127.0.0.1:8090 \
  --coordinator http://$COORD --data-dir "$TMP/relay" >/tmp/relay.log 2>&1 & PIDS+=($!)
sleep 2
RELAY_ID="$(curl -fsS http://127.0.0.1:8090/api/status | python3 -c 'import sys,json;print(json.load(sys.stdin)["peer_id"])')"
RELAY_MADDR="$RELAY_P2P/p2p/$RELAY_ID"
echo "relay peer: $RELAY_ID"
echo "relay addr: $RELAY_MADDR"

echo "=== client A (seeder) via relay ==="
RUST_LOG=info,libp2p=warn "$P" --coordinator http://$COORD --http 127.0.0.1:8080 \
  --data-dir "$TMP/a" --p2p-listen /ip4/127.0.0.1/tcp/0 --relay-addr "$RELAY_MADDR" >/tmp/a.log 2>&1 & PIDS+=($!)

echo "=== client B (leecher) via relay ==="
RUST_LOG=info,libp2p=warn "$P" --coordinator http://$COORD --http 127.0.0.1:8081 \
  --data-dir "$TMP/b" --p2p-listen /ip4/127.0.0.1/tcp/0 --relay-addr "$RELAY_MADDR" >/tmp/b.log 2>&1 & PIDS+=($!)

sleep 6

echo
echo "=== client A advertised addresses (note the /p2p-circuit relay address) ==="
curl -fsS http://127.0.0.1:8080/api/status | python3 -c 'import sys,json;d=json.load(sys.stdin);[print(" ",a) for a in d["listen_addrs"]]'

echo
echo "=== B leeches a completion from the swarm ==="
curl -fsS http://127.0.0.1:8081/v1/chat/completions -H 'content-type: application/json' \
  -d "{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Reply with exactly: swarm online\"}]}" \
  | python3 -c 'import sys,json;d=json.load(sys.stdin);print("  content :",d["choices"][0]["message"]["content"]);print("  provider:",d["p2p_provider"]);print("  tokens  :",d["usage"]["completion_tokens"])'

echo
echo "=== ledger after settle ==="
curl -fsS http://127.0.0.1:8080/api/status | python3 -c 'import sys,json;d=json.load(sys.stdin);print("  A served  :",d["served"])'
curl -fsS http://127.0.0.1:8081/api/status | python3 -c 'import sys,json;d=json.load(sys.stdin);print("  B consumed:",d["consumed"],"ratio:",d["ratio"])'

echo
echo "=== relay-node log (circuit reservations / relay events) ==="
grep -iE "listening|circuit|relay|reservation|external" /tmp/relay.log | tail -6
echo "=== client A log (relay/circuit/dcutr) ==="
grep -iE "circuit|relay|dcutr|external|listening on /ip4/127.0.0.1/tcp/4001" /tmp/a.log | tail -6
echo "=== done ==="
