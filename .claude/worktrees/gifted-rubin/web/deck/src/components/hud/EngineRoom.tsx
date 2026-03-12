import React from 'react';

interface EngineRoomProps {
  stats: any;
}

export const EngineRoom: React.FC<EngineRoomProps> = ({ stats }) => {
  return (
    <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', display: 'flex', flexDirection: 'column' }}>
      <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>ENGINE ROOM</h2>
      
      <div style={{ marginTop: '1.5rem' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-end' }}>
          <div>
            <div style={{ color: '#888', fontSize: '0.8rem' }}>CPU LOAD</div>
            <div style={{ fontSize: '2.5rem', fontWeight: 'bold', color: '#fff' }}>
              {stats ? `${stats.cpu_usage.toFixed(1)}%` : '0.0%'}
            </div>
          </div>
          <div style={{ display: 'flex', gap: '2px', height: '30px', alignItems: 'flex-end' }}>
            {stats?.history?.map((h: any, i: number) => (
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
          <div style={{ color: '#888', fontSize: '0.8rem' }}>NEURAL IMIPRINTS</div>
          <div style={{ fontSize: '1.5rem', color: '#00ffff' }}>{stats?.skill_count || 0}</div>
        </div>
        <div>
          <div style={{ color: '#888', fontSize: '0.8rem' }}>ACTIVE DIRECTIVES</div>
          <div style={{ fontSize: '1.5rem', color: '#ffa500' }}>{stats?.active_tasks || 0}</div>
        </div>
      </div>

      <div style={{ marginTop: 'auto', paddingTop: '1rem' }}>
        <div style={{ color: '#888', fontSize: '0.8rem' }}>SYSTEM UPTIME</div>
        <div style={{ fontSize: '1.2rem', color: '#fff' }}>
          {stats ? `${stats.uptime}s` : '0s'}
        </div>
      </div>
    </section>
  );
};
