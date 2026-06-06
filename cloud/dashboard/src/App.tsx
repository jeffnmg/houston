import { useState, useEffect, useCallback } from 'react'
import * as api from './api'
import type { Tenant, Agent } from './api'

// ─── Pixel sprite characters per provider ───────────────────────────────────
const SPRITES: Record<string, string[]> = {
  openai: [
    '  ████  ',
    ' ██████ ',
    '██ ██ ██',
    '████████',
    ' ██  ██ ',
    '  ████  ',
  ],
  anthropic: [
    ' ██████ ',
    '████████',
    '██ ▓▓ ██',
    '████████',
    '██████████',
    ' ██  ██ ',
  ],
  gemini: [
    '   ██   ',
    '  ████  ',
    ' ██████ ',
    '████████',
    ' ██████ ',
    '  ████  ',
  ],
  default: [
    ' ██████ ',
    '████████',
    '██ ██ ██',
    '████████',
    '████████',
    ' ██  ██ ',
  ],
}

const PROVIDER_COLORS: Record<string, string> = {
  openai: '#00bfff',
  anthropic: '#bf5fff',
  gemini: '#ffd700',
  default: '#00ff41',
}

const PROVIDER_LABELS: Record<string, string> = {
  openai: 'GPT',
  anthropic: 'Claude',
  gemini: 'Gemini',
}

function PixelSprite({ provider, running }: { provider: string; running: boolean }) {
  const lines = SPRITES[provider] || SPRITES.default
  const color = PROVIDER_COLORS[provider] || PROVIDER_COLORS.default
  const [frame, setFrame] = useState(0)

  useEffect(() => {
    if (!running) return
    const t = setInterval(() => setFrame(f => (f + 1) % 2), 600)
    return () => clearInterval(t)
  }, [running])

  return (
    <div style={{ fontFamily: 'monospace', fontSize: 8, lineHeight: '9px', color, userSelect: 'none' }}>
      {lines.map((line, i) => (
        <div key={i} style={{
          opacity: running ? (frame === 0 || i < 4 ? 1 : 0.7) : 0.3,
          letterSpacing: 0,
        }}>{line}</div>
      ))}
    </div>
  )
}

// ─── Health dot ─────────────────────────────────────────────────────────────
function StatusDot({ ok }: { ok: boolean }) {
  return (
    <span style={{
      display: 'inline-block',
      width: 8, height: 8,
      background: ok ? '#00ff41' : '#ff3b3b',
      animation: ok ? 'pulse 2s infinite' : 'none',
    }} />
  )
}

// ─── LOGIN SCREEN ────────────────────────────────────────────────────────────
function LoginScreen({ onLogin }: { onLogin: (token: string) => void }) {
  const [token, setToken] = useState('change-me-in-production')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')
  const [blink, setBlink] = useState(true)

  useEffect(() => {
    const t = setInterval(() => setBlink(b => !b), 500)
    return () => clearInterval(t)
  }, [])

  async function handleStart() {
    setLoading(true)
    setError('')
    api.setToken(token)
    const ok = await api.checkHealth()
    if (ok) {
      onLogin(token)
    } else {
      setError('CANNOT CONNECT TO CONTROL PLANE')
      setLoading(false)
    }
  }

  return (
    <div style={{
      minHeight: '100vh',
      display: 'flex',
      flexDirection: 'column',
      alignItems: 'center',
      justifyContent: 'center',
      gap: 40,
      padding: 24,
    }}>
      {/* Logo */}
      <div style={{ textAlign: 'center' }}>
        <div style={{
          fontSize: 11,
          color: '#00ff41',
          letterSpacing: 4,
          marginBottom: 8,
          fontFamily: 'monospace',
          lineHeight: '14px',
        }}>
          {[
            '██╗  ██╗ ██████╗ ██╗   ██╗███████╗████████╗ ██████╗ ███╗   ██╗',
            '██║  ██║██╔═══██╗██║   ██║██╔════╝╚══██╔══╝██╔═══██╗████╗  ██║',
            '███████║██║   ██║██║   ██║███████╗   ██║   ██║   ██║██╔██╗ ██║',
            '██╔══██║██║   ██║██║   ██║╚════██║   ██║   ██║   ██║██║╚██╗██║',
            '██║  ██║╚██████╔╝╚██████╔╝███████║   ██║   ╚██████╔╝██║ ╚████║',
            '╚═╝  ╚═╝ ╚═════╝  ╚═════╝ ╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═══╝',
          ].map((row, i) => <div key={i} style={{ fontSize: 7, lineHeight: '10px' }}>{row}</div>)}
        </div>
        <div style={{ fontSize: 9, color: '#00aa2b', marginTop: 8, letterSpacing: 3 }}>
          AGENT INFRASTRUCTURE v1.0
        </div>
      </div>

      {/* Sprites showcase */}
      <div style={{ display: 'flex', gap: 32, alignItems: 'flex-end' }}>
        {['openai', 'anthropic', 'gemini'].map(p => (
          <div key={p} style={{ textAlign: 'center' }}>
            <PixelSprite provider={p} running={true} />
            <div style={{ fontSize: 7, color: PROVIDER_COLORS[p], marginTop: 6 }}>
              {PROVIDER_LABELS[p]}
            </div>
          </div>
        ))}
      </div>

      {/* Login box */}
      <div className="px-box" style={{ width: '100%', maxWidth: 440, padding: 24, background: '#0d0d0d' }}>
        <div className="field">
          <label className="px-label">Admin Token</label>
          <input
            className="px-input"
            type="password"
            value={token}
            onChange={e => setToken(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleStart()}
            placeholder="ENTER TOKEN..."
          />
        </div>

        {error && (
          <div style={{ fontSize: 8, color: '#ff3b3b', marginBottom: 16, textAlign: 'center' }}>
            {error}
          </div>
        )}

        <button
          className="btn"
          style={{ width: '100%', fontSize: 11, padding: '14px 0' }}
          onClick={handleStart}
          disabled={loading || !token}
        >
          {loading ? 'CONNECTING...' : blink ? '▶ PRESS START' : '  PRESS START'}
        </button>
      </div>

      <div style={{ fontSize: 7, color: '#004d15', textAlign: 'center' }}>
        HOUSTON HACKCAMP 2026 · K8S MULTI-TENANT AGENT PLATFORM
      </div>
    </div>
  )
}

// ─── AGENT CARD ──────────────────────────────────────────────────────────────
function AgentCard({
  agent,
  onDelete,
}: {
  agent: Agent
  onDelete: () => void
}) {
  const [confirming, setConfirming] = useState(false)
  const color = PROVIDER_COLORS[agent.provider] || PROVIDER_COLORS.default

  return (
    <div style={{
      border: `3px solid ${color}33`,
      background: '#0d0d0d',
      padding: 16,
      display: 'flex',
      flexDirection: 'column',
      gap: 12,
      animation: 'slideIn 0.2s ease',
      position: 'relative',
    }}>
      {/* Status badge */}
      <div style={{
        position: 'absolute', top: 8, right: 8,
        display: 'flex', alignItems: 'center', gap: 4,
        fontSize: 7, color: '#00ff41',
      }}>
        <StatusDot ok={true} />
        <span>RUNNING</span>
      </div>

      {/* Sprite */}
      <div style={{ display: 'flex', justifyContent: 'center', padding: '8px 0' }}>
        <PixelSprite provider={agent.provider} running={true} />
      </div>

      {/* Name */}
      <div style={{ fontSize: 9, color, textAlign: 'center', letterSpacing: 1 }}>
        {agent.name.toUpperCase()}
      </div>

      {/* Provider badge */}
      <div style={{
        fontSize: 7,
        color: '#0a0a0a',
        background: color,
        textAlign: 'center',
        padding: '3px 0',
      }}>
        {PROVIDER_LABELS[agent.provider] || agent.provider.toUpperCase()}
      </div>

      {/* ID */}
      <div style={{ fontSize: 6, color: '#004d15', textAlign: 'center', wordBreak: 'break-all' }}>
        ID: {agent.id.slice(0, 16)}...
      </div>

      {/* Delete */}
      {confirming ? (
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn-red" style={{ flex: 1, fontSize: 7 }} onClick={onDelete}>
            CONFIRM
          </button>
          <button className="btn" style={{ flex: 1, fontSize: 7 }} onClick={() => setConfirming(false)}>
            CANCEL
          </button>
        </div>
      ) : (
        <button className="btn btn-red" style={{ fontSize: 7, width: '100%' }} onClick={() => setConfirming(true)}>
          DECOMMISSION
        </button>
      )}
    </div>
  )
}

// ─── CREATE AGENT MODAL ──────────────────────────────────────────────────────
function CreateAgentModal({
  tenant,
  onClose,
  onCreated,
}: {
  tenant: Tenant
  onClose: () => void
  onCreated: () => void
}) {
  const [name, setName] = useState('')
  const [provider, setProvider] = useState('openai')
  const [apiKey, setApiKey] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleCreate() {
    if (!name.trim()) return
    setLoading(true)
    setError('')
    try {
      await api.createAgent(tenant.id, name.trim(), provider, apiKey)
      onCreated()
      onClose()
    } catch (e: any) {
      setError(e.message || 'ERROR')
      setLoading(false)
    }
  }

  return (
    <div style={{
      position: 'fixed', inset: 0,
      background: 'rgba(0,0,0,0.85)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      zIndex: 1000,
      padding: 16,
    }}>
      <div className="px-box" style={{
        width: '100%', maxWidth: 420,
        background: '#0a0a0a',
        padding: 24,
        animation: 'slideIn 0.15s ease',
      }}>
        <div style={{ fontSize: 10, marginBottom: 20, color: '#00ff41' }}>
          ▶ DEPLOY NEW AGENT
        </div>
        <div style={{ fontSize: 7, color: '#00aa2b', marginBottom: 20 }}>
          TENANT: {tenant.name.toUpperCase()}
        </div>

        <div className="field">
          <label className="px-label">Agent Name</label>
          <input
            className="px-input"
            value={name}
            onChange={e => setName(e.target.value)}
            placeholder="my-agent-001"
            autoFocus
          />
        </div>

        <div className="field">
          <label className="px-label">LLM Provider</label>
          <select className="px-select" value={provider} onChange={e => setProvider(e.target.value)}>
            <option value="openai">GPT (OpenAI)</option>
            <option value="anthropic">Claude (Anthropic)</option>
            <option value="gemini">Gemini (Google)</option>
          </select>
        </div>

        <div className="field">
          <label className="px-label">API Key</label>
          <input
            className="px-input"
            type="password"
            value={apiKey}
            onChange={e => setApiKey(e.target.value)}
            placeholder="sk-..."
          />
          <span style={{ fontSize: 6, color: '#004d15', marginTop: 4 }}>
            STORED AS K8S SECRET · NEVER EXPOSED
          </span>
        </div>

        {error && (
          <div style={{ fontSize: 7, color: '#ff3b3b', marginBottom: 12 }}>{error}</div>
        )}

        {/* Preview sprite */}
        <div style={{
          display: 'flex', alignItems: 'center', gap: 16,
          border: `2px solid ${PROVIDER_COLORS[provider]}33`,
          padding: 12, marginBottom: 16,
        }}>
          <PixelSprite provider={provider} running={true} />
          <div>
            <div style={{ fontSize: 8, color: PROVIDER_COLORS[provider] }}>
              {name || 'UNNAMED'}
            </div>
            <div style={{ fontSize: 7, color: '#00aa2b', marginTop: 4 }}>
              {PROVIDER_LABELS[provider] || provider.toUpperCase()} AGENT
            </div>
            <div style={{ fontSize: 6, color: '#004d15', marginTop: 4 }}>
              NS: {tenant.namespace}
            </div>
          </div>
        </div>

        <div style={{ display: 'flex', gap: 8 }}>
          <button
            className="btn"
            style={{ flex: 1 }}
            onClick={handleCreate}
            disabled={loading || !name.trim()}
          >
            {loading ? 'DEPLOYING...' : '▶ DEPLOY'}
          </button>
          <button className="btn btn-red" style={{ flex: 1 }} onClick={onClose} disabled={loading}>
            CANCEL
          </button>
        </div>
      </div>
    </div>
  )
}

// ─── CREATE TENANT MODAL ─────────────────────────────────────────────────────
function CreateTenantModal({
  onClose,
  onCreated,
}: {
  onClose: () => void
  onCreated: () => void
}) {
  const [name, setName] = useState('')
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState('')

  async function handleCreate() {
    if (!name.trim()) return
    setLoading(true)
    setError('')
    try {
      await api.createTenant(name.trim())
      onCreated()
      onClose()
    } catch (e: any) {
      setError(e.message || 'ERROR')
      setLoading(false)
    }
  }

  return (
    <div style={{
      position: 'fixed', inset: 0,
      background: 'rgba(0,0,0,0.85)',
      display: 'flex', alignItems: 'center', justifyContent: 'center',
      zIndex: 1000, padding: 16,
    }}>
      <div className="px-box" style={{
        width: '100%', maxWidth: 380,
        background: '#0a0a0a', padding: 24,
        animation: 'slideIn 0.15s ease',
      }}>
        <div style={{ fontSize: 10, marginBottom: 20 }}>▶ NEW TENANT</div>

        <div className="field">
          <label className="px-label">Company Name</label>
          <input
            className="px-input"
            value={name}
            onChange={e => setName(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && handleCreate()}
            placeholder="Acme Corp"
            autoFocus
          />
        </div>

        <div style={{
          fontSize: 7, color: '#00aa2b',
          border: '2px solid #004d15',
          padding: 10, marginBottom: 16,
        }}>
          <div>K8S NAMESPACE → tenant-{name.toLowerCase().replace(/[^a-z0-9]/g, '-') || 'company'}</div>
          <div style={{ marginTop: 6 }}>NETWORK POLICY → ISOLATED</div>
          <div style={{ marginTop: 6 }}>STORAGE → 2GB PVC</div>
        </div>

        {error && <div style={{ fontSize: 7, color: '#ff3b3b', marginBottom: 12 }}>{error}</div>}

        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn" style={{ flex: 1 }} onClick={handleCreate} disabled={loading || !name.trim()}>
            {loading ? 'CREATING...' : '▶ CREATE'}
          </button>
          <button className="btn btn-red" style={{ flex: 1 }} onClick={onClose} disabled={loading}>
            CANCEL
          </button>
        </div>
      </div>
    </div>
  )
}

// ─── TENANT PANEL ─────────────────────────────────────────────────────────────
function TenantPanel({
  tenant,
  isSelected,
  onSelect,
}: {
  tenant: Tenant
  isSelected: boolean
  onSelect: () => void
}) {
  return (
    <button
      onClick={onSelect}
      style={{
        fontFamily: 'Press Start 2P',
        fontSize: 8,
        padding: '10px 14px',
        background: isSelected ? '#00ff41' : '#0d0d0d',
        color: isSelected ? '#0a0a0a' : '#00aa2b',
        border: `2px solid ${isSelected ? '#00ff41' : '#004d15'}`,
        cursor: 'pointer',
        textAlign: 'left',
        width: '100%',
        letterSpacing: 1,
      }}
    >
      {isSelected ? '▶ ' : '  '}{tenant.name.toUpperCase()}
    </button>
  )
}

// ─── MAIN DASHBOARD ──────────────────────────────────────────────────────────
function Dashboard() {
  const [tenants, setTenants] = useState<Tenant[]>([])
  const [agents, setAgents] = useState<Record<string, Agent[]>>({})
  const [selectedTenant, setSelectedTenant] = useState<Tenant | null>(null)
  const [showNewAgent, setShowNewAgent] = useState(false)
  const [showNewTenant, setShowNewTenant] = useState(false)
  const [loading, setLoading] = useState(true)
  const [lastRefresh, setLastRefresh] = useState(Date.now())

  const load = useCallback(async () => {
    try {
      const ts = await api.listTenants()
      setTenants(ts)
      const agentMap: Record<string, Agent[]> = {}
      await Promise.all(ts.map(async t => {
        try {
          agentMap[t.id] = await api.listAgents(t.id)
        } catch {
          agentMap[t.id] = []
        }
      }))
      setAgents(agentMap)
      if (!selectedTenant && ts.length > 0) setSelectedTenant(ts[0])
    } finally {
      setLoading(false)
    }
  }, [selectedTenant])

  useEffect(() => { load() }, [lastRefresh])

  // Auto-refresh every 10s
  useEffect(() => {
    const t = setInterval(() => setLastRefresh(Date.now()), 10000)
    return () => clearInterval(t)
  }, [])

  const refresh = () => setLastRefresh(Date.now())

  const totalAgents = Object.values(agents).flat().length
  const selectedAgents = selectedTenant ? (agents[selectedTenant.id] || []) : []

  async function handleDeleteAgent(tenantId: string, agentId: string) {
    try {
      await api.deleteAgent(tenantId, agentId)
      refresh()
    } catch (e: any) {
      alert(e.message)
    }
  }

  return (
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* ── Header ── */}
      <header style={{
        borderBottom: '3px solid #004d15',
        padding: '12px 24px',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'space-between',
        background: '#0a0a0a',
        gap: 16,
        flexWrap: 'wrap',
      }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: 16 }}>
          <span style={{ fontSize: 12, letterSpacing: 3 }}>HOUSTON</span>
          <span style={{ fontSize: 7, color: '#004d15' }}>AGENT CONTROL</span>
        </div>

        <div style={{ display: 'flex', gap: 24, alignItems: 'center', flexWrap: 'wrap' }}>
          <div style={{ fontSize: 7, color: '#00aa2b' }}>
            <span style={{ color: '#004d15' }}>TENANTS </span>{tenants.length}
          </div>
          <div style={{ fontSize: 7, color: '#00aa2b' }}>
            <span style={{ color: '#004d15' }}>AGENTS </span>{totalAgents}
          </div>
          <div style={{ display: 'flex', alignItems: 'center', gap: 6, fontSize: 7, color: '#00ff41' }}>
            <StatusDot ok={true} />
            <span>CLUSTER OK</span>
          </div>
        </div>
      </header>

      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* ── Sidebar: Tenants ── */}
        <aside style={{
          width: 220,
          borderRight: '3px solid #004d15',
          background: '#090909',
          display: 'flex',
          flexDirection: 'column',
          flexShrink: 0,
        }}>
          <div style={{
            padding: '12px 14px',
            borderBottom: '2px solid #004d15',
            fontSize: 7,
            color: '#004d15',
            letterSpacing: 2,
          }}>
            TENANTS
          </div>

          <div style={{ flex: 1, overflowY: 'auto', padding: 8, display: 'flex', flexDirection: 'column', gap: 4 }}>
            {loading ? (
              <div style={{ fontSize: 7, color: '#004d15', padding: 8, animation: 'pulse 1s infinite' }}>
                LOADING...
              </div>
            ) : tenants.length === 0 ? (
              <div style={{ fontSize: 7, color: '#004d15', padding: 8 }}>NO TENANTS</div>
            ) : tenants.map(t => (
              <TenantPanel
                key={t.id}
                tenant={t}
                isSelected={selectedTenant?.id === t.id}
                onSelect={() => setSelectedTenant(t)}
              />
            ))}
          </div>

          <div style={{ padding: 8, borderTop: '2px solid #004d15' }}>
            <button
              className="btn btn-yellow"
              style={{ width: '100%', fontSize: 7 }}
              onClick={() => setShowNewTenant(true)}
            >
              + NEW TENANT
            </button>
          </div>
        </aside>

        {/* ── Main: Agents ── */}
        <main style={{ flex: 1, overflow: 'auto', padding: 24 }}>
          {!selectedTenant ? (
            <div style={{
              height: '100%', display: 'flex',
              alignItems: 'center', justifyContent: 'center',
              fontSize: 9, color: '#004d15',
            }}>
              SELECT A TENANT
            </div>
          ) : (
            <>
              {/* Tenant header */}
              <div style={{
                display: 'flex', alignItems: 'center',
                justifyContent: 'space-between',
                marginBottom: 24, flexWrap: 'wrap', gap: 12,
              }}>
                <div>
                  <div style={{ fontSize: 11, marginBottom: 4 }}>
                    {selectedTenant.name.toUpperCase()}
                  </div>
                  <div style={{ fontSize: 7, color: '#004d15' }}>
                    NS: {selectedTenant.namespace} · {selectedAgents.length} AGENT{selectedAgents.length !== 1 ? 'S' : ''}
                  </div>
                </div>
                <div style={{ display: 'flex', gap: 8 }}>
                  <button className="btn" style={{ fontSize: 7 }} onClick={refresh}>
                    ↻ REFRESH
                  </button>
                  <button
                    className="btn btn-blue"
                    style={{ fontSize: 7 }}
                    onClick={() => setShowNewAgent(true)}
                  >
                    + DEPLOY AGENT
                  </button>
                </div>
              </div>

              {/* Isolation badge */}
              <div style={{
                fontSize: 7,
                border: '2px solid #004d15',
                color: '#00aa2b',
                padding: '8px 12px',
                marginBottom: 24,
                display: 'inline-flex',
                gap: 16,
              }}>
                <span>NETWORK POLICY: ISOLATED</span>
                <span>·</span>
                <span>RBAC: ENFORCED</span>
                <span>·</span>
                <span>SECRETS: ENCRYPTED</span>
              </div>

              {/* Agent grid */}
              {selectedAgents.length === 0 ? (
                <div style={{
                  border: '2px dashed #004d15',
                  padding: 48,
                  textAlign: 'center',
                  fontSize: 9,
                  color: '#004d15',
                }}>
                  <div style={{ marginBottom: 16 }}>NO AGENTS DEPLOYED</div>
                  <button
                    className="btn btn-blue"
                    onClick={() => setShowNewAgent(true)}
                  >
                    DEPLOY FIRST AGENT
                  </button>
                </div>
              ) : (
                <div style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))',
                  gap: 16,
                }}>
                  {selectedAgents.map(a => (
                    <AgentCard
                      key={a.id}
                      agent={a}
                      onDelete={() => handleDeleteAgent(selectedTenant.id, a.id)}
                    />
                  ))}
                </div>
              )}
            </>
          )}
        </main>
      </div>

      {/* ── Modals ── */}
      {showNewAgent && selectedTenant && (
        <CreateAgentModal
          tenant={selectedTenant}
          onClose={() => setShowNewAgent(false)}
          onCreated={refresh}
        />
      )}
      {showNewTenant && (
        <CreateTenantModal
          onClose={() => setShowNewTenant(false)}
          onCreated={refresh}
        />
      )}
    </div>
  )
}

// ─── ROOT ─────────────────────────────────────────────────────────────────────
export default function App() {
  const [loggedIn, setLoggedIn] = useState(false)

  return loggedIn
    ? <Dashboard />
    : <LoginScreen onLogin={() => setLoggedIn(true)} />
}
