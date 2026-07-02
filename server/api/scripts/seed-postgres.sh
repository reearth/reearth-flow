#!/usr/bin/env bash
# seed-postgres.sh runs the flow dbmigrate binary as a one-shot Cloud Run job.
#
# It applies the embedded Atlas schema to a fresh Postgres and then replicates
# the flow-owned Mongo collections into it. The job runs inside the project's
# VPC so it can reach a private-IP Cloud SQL instance without a bastion.
set -euo pipefail

JOB_NAME="reearth-flow-dbmigrate"
REGION="us-central1"
NETWORK="default"
SUBNET="default"
MONGO_SECRET="reearth-flow-db"
PG_SECRET="reearth-flow-db-postgres"
DB_NAME="reearth-flow"
DRY_RUN=false
VERIFY=false
PROJECT=""
IMAGE=""
SA=""

usage() {
  cat <<'EOF'
Usage: seed-postgres.sh --project ID --image IMAGE --service-account SA [options]

Required:
  --project ID            GCP project id
  --image IMAGE           flow-api image containing the dbmigrate binary
  --service-account SA    runtime SA (needs secretAccessor on both secrets)

Options:
  --job-name NAME         ephemeral job name          (default: reearth-flow-dbmigrate)
  --region REGION         Cloud Run region            (default: us-central1)
  --network NAME          VPC network                 (default: default)
  --subnet NAME           VPC subnet                  (default: default)
  --mongo-secret NAME     source Mongo DSN secret     (default: reearth-flow-db)
  --pg-secret NAME        target Postgres DSN secret  (default: reearth-flow-db-postgres)
  --db-name NAME          source Mongo db name        (default: reearth-flow)
  --verify                run read-back verification after replicate
  --dry-run               print the gcloud commands without executing
  -h, --help              show this help
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --project) PROJECT="$2"; shift 2 ;;
    --image) IMAGE="$2"; shift 2 ;;
    --service-account) SA="$2"; shift 2 ;;
    --job-name) JOB_NAME="$2"; shift 2 ;;
    --region) REGION="$2"; shift 2 ;;
    --network) NETWORK="$2"; shift 2 ;;
    --subnet) SUBNET="$2"; shift 2 ;;
    --mongo-secret) MONGO_SECRET="$2"; shift 2 ;;
    --pg-secret) PG_SECRET="$2"; shift 2 ;;
    --db-name) DB_NAME="$2"; shift 2 ;;
    --verify) VERIFY=true; shift ;;
    --dry-run) DRY_RUN=true; shift ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage; exit 2 ;;
  esac
done

if [ -z "$PROJECT" ] || [ -z "$IMAGE" ] || [ -z "$SA" ]; then
  echo "error: --project, --image and --service-account are required" >&2
  usage
  exit 2
fi

run() {
  if [ "$DRY_RUN" = true ]; then
    printf 'DRY-RUN:'
    printf ' %q' "$@"
    printf '\n'
  else
    "$@"
  fi
}

ARGS="-apply-schema,-db=${DB_NAME}"
if [ "$VERIFY" = true ]; then
  ARGS="${ARGS},-verify"
fi

cleanup() {
  run gcloud run jobs delete "$JOB_NAME" \
    --project "$PROJECT" --region "$REGION" --quiet || true
}
trap cleanup EXIT

run gcloud run jobs create "$JOB_NAME" \
  --project "$PROJECT" --region "$REGION" \
  --image "$IMAGE" \
  --network "$NETWORK" --subnet "$SUBNET" --vpc-egress private-ranges-only \
  --service-account "$SA" \
  --set-secrets "REEARTH_FLOW_DB=${MONGO_SECRET}:latest,REEARTH_FLOW_DB_PG=${PG_SECRET}:latest" \
  --command /reearth-flow/dbmigrate \
  --args "$ARGS" \
  --max-retries 0 --task-timeout 3600s

run gcloud run jobs execute "$JOB_NAME" \
  --project "$PROJECT" --region "$REGION" --wait
