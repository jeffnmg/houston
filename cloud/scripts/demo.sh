#!/usr/bin/env bash
set -euo pipefail

API="${HOUSTON_CP_URL:-http://localhost:8080}"
TOKEN="${HOUSTON_CP_ADMIN_TOKEN:-change-me-in-production}"
AUTH=(-H "Authorization: Bearer ${TOKEN}" -H "Content-Type: application/json")

echo "==> Create tenant Acme"
ACME=$(curl -sf "${AUTH[@]}" -d '{"name":"Acme Corp"}' "${API}/v1/tenants")
ACME_ID=$(echo "$ACME" | jq -r .id)
echo "$ACME" | jq .

echo "==> Create tenant Globex"
GLOBEX=$(curl -sf "${AUTH[@]}" -d '{"name":"Globex Inc"}' "${API}/v1/tenants")
GLOBEX_ID=$(echo "$GLOBEX" | jq -r .id)
echo "$GLOBEX" | jq .

if [[ -n "${ANTHROPIC_API_KEY:-}" ]]; then
  echo "==> Deploy Acme agent (anthropic)"
  curl -sf "${AUTH[@]}" \
    -d "{\"name\":\"HR Assistant\",\"provider\":\"anthropic\",\"api_key\":\"${ANTHROPIC_API_KEY}\"}" \
    "${API}/v1/tenants/${ACME_ID}/agents" | jq .
fi

if [[ -n "${OPENAI_API_KEY:-}" ]]; then
  echo "==> Deploy Globex agent (openai)"
  curl -sf "${AUTH[@]}" \
    -d "{\"name\":\"Sales Assistant\",\"provider\":\"openai\",\"api_key\":\"${OPENAI_API_KEY}\"}" \
    "${API}/v1/tenants/${GLOBEX_ID}/agents" | jq .
fi

echo ""
echo "Tenants:"
curl -sf "${AUTH[@]}" "${API}/v1/tenants" | jq .
echo ""
echo "Acme agents:"
curl -sf "${AUTH[@]}" "${API}/v1/tenants/${ACME_ID}/agents" | jq .
echo ""
echo "Globex agents:"
curl -sf "${AUTH[@]}" "${API}/v1/tenants/${GLOBEX_ID}/agents" | jq .

echo ""
echo "Export for isolation test:"
echo "  export DEMO_ACME_TENANT=${ACME_ID}"
echo "  export DEMO_GLOBEX_TENANT=${GLOBEX_ID}"
