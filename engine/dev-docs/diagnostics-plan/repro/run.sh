#!/usr/bin/env bash
# Run a workflow through the worker locally and print the diagnostics artifact.
#
#   ./run.sh expr-fatal.yml
#
# Requires: cargo build -p reearth-flow-worker --bin reearth-flow-worker   (run from engine/)
# No GCP, no MongoDB, no pubsub — `--pubsub-backend noop` covers all of it.
set -euo pipefail
HERE="$(cd "$(dirname "$0")" && pwd)"
ENGINE="$(cd "$HERE/../../.." && pwd)"          # .../engine
WF="${1:?usage: run.sh <workflow.yml>}"
BIN="$ENGINE/target/debug/reearth-flow-worker"

[ -x "$BIN" ] || { echo "worker not built. from $ENGINE run:"; \
  echo "  cargo build -p reearth-flow-worker --bin reearth-flow-worker"; exit 1; }

JOB=$(python3 -c "import uuid;print(uuid.uuid4())")
WORK="$(mktemp -d)"
mkdir -p "$WORK/artifacts"
cat > "$WORK/metadata.json" <<EOF
{
  "jobId": "$JOB",
  "assets": { "baseUrl": "file://$WORK/artifacts", "files": [] },
  "artifactBaseUrl": "file://$WORK/artifacts",
  "timestamps": { "created": "2026-01-01T00:00:00Z" }
}
EOF

echo "=== job $JOB ==="
"$BIN" --workflow "$WF" --metadata-path "file://$WORK/metadata.json" \
       --pubsub-backend noop >"$WORK/run.log" 2>&1 || true

# macOS cache dir; on Linux this is ~/.cache/reearth.flow.workers
CACHE="$HOME/Library/Caches/reearth.flow.workers/jobs/$JOB"
[ -d "$CACHE" ] || CACHE="$HOME/.cache/reearth.flow.workers/jobs/$JOB"

echo
echo "=== how many times did the action actually fail? ==="
grep -c "Error operation" "$CACHE/worker/worker.log" 2>/dev/null || true
echo
echo "=== diagnostics.json (what the frontend eventually receives) ==="
python3 -m json.tool "$CACHE/diagnostics.json" 2>/dev/null || echo "(no diagnostics artifact — run.log: $WORK/run.log)"
echo
# The data the workflow actually wrote. A diagnostics payload can look correct while the
# output is silently wrong (a demoted failure once emitted a fabricated 0), so always check
# both before calling a diagnostics change verified.
echo "=== output data the workflow wrote ==="
cat "$WORK/artifacts/out.json" 2>/dev/null || cat "$CACHE/artifacts/out.json" 2>/dev/null \
  || echo "(no output file — no features reached the writer)"
echo
echo "artifact : $CACHE/diagnostics.json"
echo "full log : $CACHE/worker/worker.log"
