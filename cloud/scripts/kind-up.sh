#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
CLUSTER_NAME="${KIND_CLUSTER_NAME:-houston-demo}"
ENGINE_IMAGE="${ENGINE_IMAGE:-houston/engine:dev}"
CP_IMAGE="${CP_IMAGE:-houston/control-plane:dev}"

echo "==> Creating kind cluster '${CLUSTER_NAME}' (skip if exists)"
if ! kind get clusters 2>/dev/null | grep -qx "${CLUSTER_NAME}"; then
  kind create cluster --name "${CLUSTER_NAME}" --config "${ROOT}/cloud/k8s/kind-config.yaml"
fi

echo "==> Building engine image"
docker build -t "${ENGINE_IMAGE}" -f "${ROOT}/always-on/Dockerfile" "${ROOT}"

echo "==> Building control plane image"
docker build -t "${CP_IMAGE}" -f "${ROOT}/cloud/control-plane/Dockerfile" "${ROOT}/cloud/control-plane"

echo "==> Loading images into kind"
kind load docker-image "${ENGINE_IMAGE}" --name "${CLUSTER_NAME}"
kind load docker-image "${CP_IMAGE}" --name "${CLUSTER_NAME}"

echo "==> Applying control plane manifests"
kubectl apply -k "${ROOT}/cloud/k8s/overlays/kind"

echo "==> Waiting for control plane"
kubectl -n houston-system rollout status deploy/houston-control-plane --timeout=120s

echo ""
echo "Control plane ready at http://localhost:8080"
echo "Admin token: change-me-in-production (override secret before prod)"
echo ""
echo "Try:"
echo "  curl -H 'Authorization: Bearer change-me-in-production' http://localhost:8080/v1/health"
