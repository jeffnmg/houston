# Houston Cloud — Hackathon MVP

Secure multi-tenant Kubernetes infrastructure for [Houston Engine](https://github.com/gethouston/houston).

**One pod = one agent. One namespace = one tenant.** Implements a slice of the official [cloud-design](https://github.com/gethouston/houston/tree/main/cloud-design) roadmap (C2 + control plane v0 + multi-tenant isolation).

## Problem

Running many agents (Houston, OpenClaw-style workloads) on a shared VPS shares the kernel and filesystem. Prompt injection or a compromised agent can lateral-move to other tenants.

## Solution

- **Control plane** (Rust): provisions tenants, deploys isolated agent pods, proxies to `houston-engine`
- **Per-tenant namespace** + **NetworkPolicy**: blocks cross-tenant traffic
- **Per-agent PVC**: dedicated `.houston/` state
- **Path to scale**: Kata/Firecracker + Knative documented in `cloud-design/` (not in this 6h MVP)

## Prerequisites

- **Docker Desktop running** (required for kind + image builds)
- [kind](https://kind.sigs.k8s.io/) (`kind-windows-amd64.exe` on PATH)
- kubectl, curl, jq (Git Bash or WSL for `.sh` scripts; or use `kind-up.ps1` on Windows)
- Optional: `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` for live LLM agents

### Windows

```powershell
# Start Docker Desktop first, then:
.\cloud\scripts\kind-up.ps1
```

## Quick start (local)

```bash
# 1. Fork this repo on GitHub, clone YOUR fork, then:
git checkout hackathon/cloud-k8s-mvp

# 2. Start cluster + control plane (~5–15 min first engine build)
bash cloud/scripts/kind-up.sh

# 3. Create demo tenants + agents
export ANTHROPIC_API_KEY=sk-...
export OPENAI_API_KEY=sk-...
bash cloud/scripts/demo.sh

# 4. Prove isolation
export DEMO_ACME_TENANT=...   # printed by demo.sh
export DEMO_GLOBEX_TENANT=...
bash cloud/scripts/prove-isolation.sh
```

Control plane: **http://localhost:8080**  
Default admin token: `change-me-in-production` (see `cloud/k8s/base/control-plane.yaml`)

## API (for Lovable / judges)

All routes require `Authorization: Bearer <admin-token>`.

| Method | Path | Description |
|--------|------|-------------|
| GET | `/v1/health` | Control plane health |
| POST | `/v1/tenants` | `{ "name": "Acme Corp" }` |
| GET | `/v1/tenants` | List tenants |
| POST | `/v1/tenants/:id/agents` | `{ "name", "provider", "api_key?" }` |
| GET | `/v1/tenants/:id/agents` | List agents |
| DELETE | `/v1/tenants/:id/agents/:aid` | Remove agent |
| * | `/v1/tenants/:id/agents/:aid/engine/*` | Proxy to houston-engine |

**Providers:** `anthropic`, `openai`, `gemini`

**Example — health via proxy:**

```bash
curl -H "Authorization: Bearer change-me-in-production" \
  "http://localhost:8080/v1/tenants/$TENANT/agents/$AGENT/engine/v1/health"
```

## Architecture

```
Judge / Lovable UI
        │
        ▼
houston-control-plane (houston-system)
        │  kube-rs provisions
        ├── tenant-acme/ns ── Pod houston-engine + PVC
        └── tenant-globex/ns ── Pod houston-engine + PVC
              NetworkPolicy: no cross-tenant ingress
```

## Repo layout

```
cloud/
├── control-plane/     Rust axum + kube-rs + SQLite
├── k8s/               Kustomize base + kind overlay
├── scripts/           kind-up, demo, prove-isolation
└── README.md          this file
```

## Demo script (3 min Loom)

1. Problem: shared VPS agents are unsafe
2. `./cloud/scripts/kind-up.sh` + `./cloud/scripts/demo.sh`
3. Chat proxy: `curl .../engine/v1/health`
4. `./cloud/scripts/prove-isolation.sh`
5. `kubectl get pods -A` — show namespaces
6. Mention Kata + Knative for 10k agents (cloud-design ch. 3–4)

## Cloud deploy (backup for judges)

Build/push images to GHCR, point `ENGINE_IMAGE` / `CP_IMAGE` in manifests, apply `cloud/k8s/base` with a LoadBalancer Service patch.

## Out of scope (documented, not built)

- Kata + Firecracker microVMs
- Knative scale-to-zero
- Postgres / Redis / Supabase SSO
- Full web UI (use Lovable against this API)

## License

MIT — same as Houston monorepo.
