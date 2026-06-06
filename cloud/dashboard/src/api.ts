const BASE = '/api'

export interface Tenant {
  id: string
  name: string
  namespace: string
  created_at: string
}

export interface Agent {
  id: string
  name: string
  tenant_id: string
  namespace: string
  provider: string
  service_name: string
  engine_url: string
}

let _token = ''
export const setToken = (t: string) => { _token = t }
export const getToken = () => _token

const headers = () => ({
  'Authorization': `Bearer ${_token}`,
  'Content-Type': 'application/json',
})

export async function checkHealth(): Promise<boolean> {
  try {
    const r = await fetch(`${BASE}/v1/health`, { headers: headers() })
    return r.ok
  } catch {
    return false
  }
}

export async function listTenants(): Promise<Tenant[]> {
  const r = await fetch(`${BASE}/v1/tenants`, { headers: headers() })
  if (!r.ok) throw new Error(await r.text())
  return r.json()
}

export async function createTenant(name: string): Promise<Tenant> {
  const r = await fetch(`${BASE}/v1/tenants`, {
    method: 'POST',
    headers: headers(),
    body: JSON.stringify({ name }),
  })
  if (!r.ok) throw new Error(await r.text())
  return r.json()
}

export async function listAgents(tenantId: string): Promise<Agent[]> {
  const r = await fetch(`${BASE}/v1/tenants/${tenantId}/agents`, { headers: headers() })
  if (!r.ok) throw new Error(await r.text())
  return r.json()
}

export async function createAgent(
  tenantId: string,
  name: string,
  provider: string,
  apiKey: string,
): Promise<Agent> {
  const r = await fetch(`${BASE}/v1/tenants/${tenantId}/agents`, {
    method: 'POST',
    headers: headers(),
    body: JSON.stringify({ name, provider, api_key: apiKey }),
  })
  if (!r.ok) throw new Error(await r.text())
  return r.json()
}

export async function deleteAgent(tenantId: string, agentId: string): Promise<void> {
  const r = await fetch(`${BASE}/v1/tenants/${tenantId}/agents/${agentId}`, {
    method: 'DELETE',
    headers: headers(),
  })
  if (!r.ok) throw new Error(await r.text())
}
