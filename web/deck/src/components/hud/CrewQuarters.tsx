import React from 'react';

interface CrewQuartersProps {
  agents: any[];
}

export const CrewQuarters: React.FC<CrewQuartersProps> = ({ agents }) => {
  return (
    <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', height: '100%', overflowY: 'auto' }}>
      <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>CREW QUARTERS [MANIFEST]</h2>
      <div style={{ marginTop: '1rem', display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '1rem' }}>
        {agents.length === 0 && <div style={{ color: '#444', fontStyle: 'italic' }}>No personnel detected.</div>}
        {agents.map(a => (
          <div key={a.session_id} style={{ padding: '1rem', border: '1px solid #222', background: '#050505' }}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
              <div>
                <div style={{ fontWeight: 'bold', color: '#00ff00', fontSize: '1.2rem' }}>{a.identity.name}</div>
                <div style={{ color: '#888', fontSize: '0.8rem' }}>{a.identity.rank.toUpperCase()} [TIER {a.identity.tier}]</div>
              </div>
              <div style={{ 
                padding: '2px 8px', 
                background: a.status === 'active' ? '#004400' : '#440000', 
                color: a.status === 'active' ? '#00ff00' : '#ff0000',
                fontSize: '0.7rem',
                border: '1px solid currentColor'
              }}>
                {a.status.toUpperCase()}
              </div>
            </div>
            <div style={{ marginTop: '1rem', fontSize: '0.8rem', color: '#aaa', borderTop: '1px solid #111', paddingTop: '0.5rem' }}>
              <div>BIO: {a.metadata.bio}</div>
              <div style={{ marginTop: '0.5rem', color: '#555' }}>SESSION: {a.session_id}</div>
            </div>
          </div>
        ))}
      </div>
    </section>
  );
};
