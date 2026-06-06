#Requires -Version 5.1
$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$ClusterName = if ($env:KIND_CLUSTER_NAME) { $env:KIND_CLUSTER_NAME } else { "houston-demo" }
$EngineImage = if ($env:ENGINE_IMAGE) { $env:ENGINE_IMAGE } else { "houston/engine:dev" }
$CpImage = if ($env:CP_IMAGE) { $env:CP_IMAGE } else { "houston/control-plane:dev" }

Write-Host "==> Ensure Docker Desktop is running"
docker info | Out-Null

$kind = Get-Command kind -ErrorAction SilentlyContinue
if (-not $kind) {
    throw "kind not found. Install from https://kind.sigs.k8s.io/ or: go install sigs.k8s.io/kind@latest"
}

$clusters = kind get clusters 2>$null
if ($clusters -notcontains $ClusterName) {
    kind create cluster --name $ClusterName --config "$Root\cloud\k8s\kind-config.yaml"
}

Write-Host "==> Building images (first run may take 15+ min)"
docker build -t $EngineImage -f "$Root\always-on\Dockerfile" $Root
docker build -t $CpImage -f "$Root\cloud\control-plane\Dockerfile" "$Root\cloud\control-plane"

kind load docker-image $EngineImage --name $ClusterName
kind load docker-image $CpImage --name $ClusterName

kubectl apply -k "$Root\cloud\k8s\overlays\kind"
kubectl -n houston-system rollout status deploy/houston-control-plane --timeout=120s

Write-Host ""
Write-Host "Control plane: http://localhost:8080"
Write-Host "Admin token: change-me-in-production"
