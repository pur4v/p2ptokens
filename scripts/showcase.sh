#!/usr/bin/env bash
# Full showcase of everything built: coordinator + relay + two unified clients,
# swarm registry, relay circuit addrs, model list, completions (both directions,
# stream + non-stream), ratio ledger, and the dashboard.
set -uo pipefail
cd "$(dirname "$0")/.."

MODEL="${MODEL:-llama3.2:3b}"
COORD=127.0.0.1:4000
RELAY_P2P=/ip4/127.0.0.1/tcp/4001
TMP="$(mktemp -d)"
PIDS=()
cleanup(){ echo; echo "--- shutdown ---"; for p in "${PIDS[@]:-}"; do kill "$p" 2>/dev/null || true; done; }
trap cleanup EXIT
hr(){ printf '\n\033[1m== %s ==\033[0m\n' "$1"; }

C=target/debug/p2p-coordinator
P=target/debug/p2ptokens

hr "starting coordinator + relay + client A (seeder) + client B (seeder/leecher)"
"$C" --listen $COORD >/tmp/coord.log 2>&1 & PIDS+=($!)
sleep 1
P2P_OLLAMA=0 "$P" --relay --p2p-listen $RELAY_P2P --http 127.0.0.1:8090 \
  --coordinator http://$COORD --data-dir "$TMP/relay" >/tmp/relay.log 2>&1 & PIDS+=($!)
sleep 2
RID="$(curl -fsS http://127.0.0.1:8090/api/status | python3 -c 'import sys,json;print(json.load(sys.stdin)["peer_id"])')"
RM="$RELAY_P2P/p2p/$RID"
"$P" --coordinator http://$COORD --http 127.0.0.1:8080 --data-dir "$TMP/a" \
  --p2p-listen /ip4/127.0.0.1/tcp/0 --relay-addr "$RM" >/tmp/a.log 2>&1 & PIDS+=($!)
"$P" --coordinator http://$COORD --http 127.0.0.1:8081 --data-dir "$TMP/b" \
  --p2p-listen /ip4/127.0.0.1/tcp/0 --relay-addr "$RM" >/tmp/b.log 2>&1 & PIDS+=($!)
sleep 6
echo "coordinator: http://$COORD   relay: ${RID:0:16}…   dashboards: :8080 (A) :8081 (B)"

hr "1) swarm registry (coordinator /providers)"
curl -fsS http://$COORD/providers | python3 -c '
import sys,json
for p in json.load(sys.stdin):
    models=[o["model"]["name"] for o in p["offers"]][:3]
    print("  {}...  cap={} inflight={}  models={}".format(p["peer_id"][:16], p["capacity"], p["in_flight"], models))'

hr "2) node A status (identity, offers, relay circuit address)"
curl -fsS http://127.0.0.1:8080/api/status | python3 -c '
import sys,json;d=json.load(sys.stdin)
print("  peer     :",d["peer_id"])
print("  ratio    :",d["ratio"],"| served",d["served"],"| consumed",d["consumed"])
print("  offers   :",[o["model"] for o in d["offers"]][:4],"…")
print("  addrs    :")
for a in d["listen_addrs"]: print("     ",a)'

hr "3) network model list (GET /v1/models)"
curl -fsS http://127.0.0.1:8081/v1/models | python3 -c 'import sys,json;print("  ",[m["id"] for m in json.load(sys.stdin)["data"]])'

hr "4) B leeches from the swarm (non-streaming)"
curl -fsS http://127.0.0.1:8081/v1/chat/completions -H 'content-type: application/json' \
  -d "{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Reply with exactly: swarm online\"}]}" \
  | python3 -c 'import sys,json;d=json.load(sys.stdin);print("  content :",d["choices"][0]["message"]["content"]);print("  served-by:",d["p2p_provider"][:16],"…  tokens:",d["usage"]["completion_tokens"])'

hr "5) A leeches from the swarm too (bidirectional barter)"
curl -fsS http://127.0.0.1:8080/v1/chat/completions -H 'content-type: application/json' \
  -d "{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Name one color, one word.\"}]}" \
  | python3 -c 'import sys,json;d=json.load(sys.stdin);print("  content :",d["choices"][0]["message"]["content"]);print("  served-by:",d["p2p_provider"][:16],"…  tokens:",d["usage"]["completion_tokens"])'

hr "6) streaming (SSE chat.completion.chunk) via B"
curl -sN http://127.0.0.1:8081/v1/chat/completions -H 'content-type: application/json' \
  -d "{\"model\":\"$MODEL\",\"stream\":true,\"messages\":[{\"role\":\"user\",\"content\":\"Count one to three.\"}]}" \
  | grep '^data:' | head -6 | sed 's/^/  /'

hr "7) ratio ledger after settlement"
for port in 8080 8081; do
  curl -fsS http://127.0.0.1:$port/api/status | python3 -c '
import sys,json
d=json.load(sys.stdin)
print("  {}...  served={}  consumed={}  ratio={}  reputation={}".format(d["peer_id"][:16], d["served"], d["consumed"], d["ratio"], d["reputation"]))'
done

hr "8) dashboard serves (GET / — ASCII torrent UI)"
curl -fsS http://127.0.0.1:8081/ | grep -iE "<title>|p2ptokens // swarm|seed to leech" | head -3 | sed 's/^/  /'
echo "  (open http://127.0.0.1:8081 in a browser for the live phosphor-green dashboard)"
