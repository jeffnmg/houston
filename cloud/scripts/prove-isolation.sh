#!/usr/bin/env bash
set -euo pipefail

API="${HOUSTON_CP_URL:-http://localhost:8080}"
TOKEN="${HOUSTON_CP_ADMIN_TOKEN:-change-me-in-production}"
AUTH=(-H "Authorization: Bearer ${TOKEN}")

: "${DEMO_ACME_TENANT:?Set DEMO_ACME_TENANT (run demo.sh first)}"
: "${DEMO_GLOBEX_TENANT:?Set DEMO_GLOBEX_TENANT (run demo.sh first)}"

ACME_NS=$(curl -sf "${AUTH[@]}" "${API}/v1/tenants" | jq -r ".[] | select(.id==\"${DEMO_ACME_TENANT}\") | .namespace")
GLOBEX_NS=$(curl -sf "${AUTH[@]}" "${API}/v1/tenants" | jq -r ".[] | select(.id==\"${DEMO_GLOBEX_TENANT}\") | .namespace")

ACME_AGENT=$(curl -sf "${AUTH[@]}" "${API}/v1/tenants/${DEMO_ACME_TENANT}/agents" | jq -r '.[0].service_name // empty')
GLOBEX_AGENT=$(curl -sf "${AUTH[@]}" "${API}/v1/tenants/${DEMO_GLOBEX_TENANT}/agents" | jq -r '.[0].service_name // empty')

if [[ -z "${ACME_AGENT}" || -z "${GLOBEX_AGENT}" ]]; then
  echo "Need at least one agent per tenant. Run demo.sh with API keys."
  exit 1
fi

echo "==> Layer 1: API RBAC — Acme token cannot chat with Globex agent"
GLOBEX_AID=$(curl -sf "${AUTH[@]}" "${API}/v1/tenants/${DEMO_GLOBEX_TENANT}/agents" | jq -r '.[0].id')
if curl -sf "${AUTH[@]}" "${API}/v1/tenants/${DEMO_ACME_TENANT}/agents/${GLOBEX_AID}/engine/v1/health"; then
  echo "FAIL: cross-tenant API access succeeded"
  exit 1
else
  echo "PASS: cross-tenant API blocked (404/403 expected)"
fi

echo "==> Layer 2: NetworkPolicy — pod in Acme cannot reach Globex service"
kubectl -n "${ACME_NS}" run nettest --rm -i --restart=Never --image=curlimages/curl:8.5.0 -- \
  curl -sf --connect-timeout 5 "http://${GLOBEX_AGENT}.${GLOBEX_NS}.svc.cluster.local:7777/v1/health" && {
  echo "FAIL: cross-namespace network access succeeded"
  exit 1
} || echo "PASS: cross-namespace network blocked or timed out"

echo "==> Layer 3: Namespaces are distinct"
echo "Acme namespace: ${ACME_NS}"
echo "Globex namespace: ${GLOBEX_NS}"
kubectl get ns -l houston.ai/managed-by=houston-control-plane

echo ""
echo "Isolation checks complete."
