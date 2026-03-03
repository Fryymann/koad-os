import React from 'react'
import { useKoadFabric } from './hooks/useKoadFabric'
import { CommandConsole } from './components/CommandConsole'

const App: React.FC = () => {
  const { stats, logs, agents, issues, projects, sendCommand } = useKoadFabric();
  const [filter, setFilter] = React.useState('');

  return (
    <div style={{
      backgroundColor: '#050505',
      color: '#00ff00',
      minHeight: '100vh',
      padding: '2rem',
      fontFamily: 'monospace'
    }}>
      <header style={{ borderBottom: '2px solid #00ff00', marginBottom: '2rem' }}>
        <h1 style={{ letterSpacing: '4px' }}>KOADOS v3 // COMMAND DECK [VITE]</h1>
      </header>

      <main style={{ display: 'grid', gridTemplateColumns: '1fr 1fr 1.5fr', gap: '2rem', height: '80vh' }}>
        {/* Engine Panel */}
        <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', display: 'flex', flexDirection: 'column' }}>
          <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>CORE ENGINE</h2>
          
          <div style={{ marginTop: '1.5rem' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end' }}>
              <div>
                <div style={{ color: '#888', fontSize: '0.8rem' }}>CPU LOAD</div>
                <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: '#fff' }}>
                  {stats ? `${stats.cpu_usage.toFixed(1)}%` : '0.0%'}
                </div>
              </div>
              {/* Sparkline */}
              <div style={{ display: 'flex', gap: '2px', height: '30px', alignItems: 'flex-end' }}>
                {stats?.history?.map((h, i) => (
                  <div key={i} style={{ width: '4px', height: `${h.cpu_usage}%`, background: '#00ff00', opacity: 0.5 + (i / 40) }} />
                ))}
              </div>
            </div>
          </div>

          <div style={{ marginTop: '1.5rem' }}>
            <div style={{ color: '#888', fontSize: '0.8rem' }}>MEMORY</div>
            <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: '#fff' }}>
              {stats ? `${stats.memory_usage} MB` : '0 MB'}
            </div>
          </div>

          <div style={{ marginTop: '1.5rem', display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
            <div>
              <div style={{ color: '#888', fontSize: '0.8rem' }}>SKILLS</div>
              <div style={{ fontSize: '1.5rem', color: '#00ffff' }}>{stats?.skill_count || 0}</div>
            </div>
            <div>
              <div style={{ color: '#888', fontSize: '0.8rem' }}>TASKS</div>
              <div style={{ fontSize: '1.5rem', color: '#ffa500' }}>{stats?.active_tasks || 0}</div>
            </div>
          </div>

          <div style={{ marginTop: 'auto', paddingTop: '1rem' }}>
            <div style={{ color: '#888', fontSize: '0.8rem' }}>UPTIME</div>
            <div style={{ fontSize: '1.2rem', color: '#fff' }}>
              {stats ? `${stats.uptime}s` : '0s'}
            </div>
          </div>
        </section>

        {/* Fabric Panel */}
        <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', overflowY: 'auto', display: 'flex', flexDirection: 'column', gap: '2rem' }}>
          <div>
            <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>MASTER PROJECT MAP</h2>
            <div style={{ marginTop: '1rem' }}>
              {projects.length === 0 && <div style={{ color: '#444', fontStyle: 'italic' }}>No projects registered.</div>}
              {projects.map(p => (
                <div key={p.id} style={{ padding: '0.5rem', border: '1px solid #222', marginBottom: '0.5rem', fontSize: '0.8rem' }}>
                  <div style={{ display: 'flex', justifyContent: 'space-between', fontWeight: 'bold' }}>
                    <span style={{ color: '#fff' }}>{p.name}</span>
                    <span style={{ color: p.health === 'green' ? '#00ff00' : p.health === 'yellow' ? '#ffa500' : p.health === 'red' ? '#ff0000' : '#888' }}>
                      [{p.health.toUpperCase()}]
                    </span>
                  </div>
                  <div style={{ color: '#888', marginTop: '0.2rem' }}>Branch: <span style={{ color: '#00ffff' }}>{p.branch}</span></div>
                </div>
              ))}
            </div>
          </div>

          <div>
            <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>ACTIVE FABRIC</h2>
            <div style={{ marginTop: '1rem' }}>
              <h3 style={{ fontSize: '0.8rem', color: '#888' }}>AGENTS IN SESSION</h3>
              {agents.length === 0 && <div style={{ color: '#444', fontStyle: 'italic' }}>No active sessions.</div>}
              {agents.map(a => (
                <div key={a.session_id} style={{ padding: '0.5rem', border: '1px solid #222', marginBottom: '0.5rem' }}>
                  <div style={{ fontWeight: 'bold', color: '#00ff00' }}>
                    {a.identity.name} <span style={{ color: '#888', fontWeight: 'normal' }}>({a.identity.rank})</span>
                  </div>
                  <div style={{ fontSize: '0.7rem', color: '#555' }}>ID: {a.session_id.slice(0,8)}... | {a.environment}</div>
                </div>
              ))}
            </div>

            <div style={{ marginTop: '2rem' }}>
              <h3 style={{ fontSize: '0.8rem', color: '#888' }}>COMMAND DECK (ISSUES)</h3>
              {issues.length === 0 && <div style={{ color: '#444', fontStyle: 'italic' }}>No issues found.</div>}
              {issues.map(i => (
                <div key={i.id} style={{ padding: '0.5rem', border: '1px solid #222', marginBottom: '0.5rem', fontSize: '0.8rem' }}>
                  <div style={{ fontWeight: 'bold' }}>
                    {i.number && <span style={{ color: '#00ffff', marginRight: '0.5rem' }}>#{i.number}</span>}
                    {i.title}
                  </div>
                  <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '0.3rem' }}>
                    <span style={{ color: i.status === 'Done' ? '#00ff00' : '#ffa500' }}>[{i.status.toUpperCase()}]</span>
                    {i.target_version && <span style={{ color: '#888' }}>{i.target_version}</span>}
                  </div>
                </div>
              ))}
            </div>
          </div>
        </section>

        {/* Telemetry Panel */}
        <section style={{ 
          border: '1px solid #00ff00', 
          padding: '1.5rem', 
          background: '#000',
          display: 'flex',
          flexDirection: 'column'
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid #222' }}>
            <h2 style={{ color: '#00ffff', margin: 0 }}>SPINE COMMS</h2>
            <input 
              type="text" 
              placeholder="FILTER LOGS..." 
              onChange={(e) => setFilter(e.target.value)}
              style={{ background: 'transparent', border: '1px solid #333', color: '#888', padding: '2px 5px', fontSize: '0.7rem', outline: 'none' }}
            />
          </div>
          <div style={{ flexGrow: 1, overflowY: 'auto', marginTop: '1rem', display: 'flex', flexDirection: 'column-reverse' }}>
            <div>
              {logs.filter(log => !filter || log.message.toLowerCase().includes(filter.toLowerCase()) || log.source.toLowerCase().includes(filter.toLowerCase())).map((log, i) => (
                <div key={i} style={{ borderBottom: '1px solid #111', padding: '0.4rem 0', fontSize: '0.9rem' }}>
                  <span style={{ color: '#555', marginRight: '1rem' }}>[{new Date(log.timestamp * 1000).toLocaleTimeString()}]</span>
                  <span style={{ color: '#00ffff', fontWeight: 'bold', marginRight: '0.5rem' }}>{log.source}</span>
                  {log.message}
                </div>
              ))}
            </div>
          </div>
          <CommandConsole onSend={sendCommand} />
        </section>
      </main>
    </div>
  )
}

export default App
