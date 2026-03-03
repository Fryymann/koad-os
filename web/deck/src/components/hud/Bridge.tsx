import React from 'react';

interface BridgeProps {
  projects: any[];
  issues: any[];
}

export const Bridge: React.FC<BridgeProps> = ({ projects, issues }) => {
  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '2rem', height: '100%' }}>
      {/* Sector Scan: Hull Integrity */}
      <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', overflowY: 'auto' }}>
        <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>SECTOR SCAN [HULL]</h2>
        <div style={{ marginTop: '1rem' }}>
          {projects.length === 0 && <div style={{ color: '#444', fontStyle: 'italic' }}>No sectors mapped.</div>}
          {projects.map(p => (
            <div key={p.id} style={{ padding: '0.5rem', border: '1px solid #222', marginBottom: '0.5rem', fontSize: '0.8rem' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', fontWeight: 'bold' }}>
                <span style={{ color: '#fff' }}>{p.name}</span>
                <span style={{ color: p.health === 'green' ? '#00ff00' : p.health === 'yellow' ? '#ffa500' : p.health === 'red' ? '#ff0000' : '#888' }}>
                  [{p.health.toUpperCase()}]
                </span>
              </div>
              <div style={{ color: '#888', marginTop: '0.2rem' }}>Vector: <span style={{ color: '#00ffff' }}>{p.branch}</span></div>
            </div>
          ))}
        </div>
      </section>

      {/* Mission Log: Directives */}
      <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', overflowY: 'auto' }}>
        <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>MISSION LOG [DIRECTIVES]</h2>
        <div style={{ marginTop: '1rem' }}>
          {issues.length === 0 && <div style={{ color: '#444', fontStyle: 'italic' }}>Mission log clear.</div>}
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
      </section>
    </div>
  );
};
