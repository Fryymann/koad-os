import React from 'react';
import { CommandConsole } from '../CommandConsole';

interface EngineeringProps {
  logs: any[];
  sendCommand: (cmd: string) => void;
}

export const Engineering: React.FC<EngineeringProps> = ({ logs, sendCommand }) => {
  const [filter, setFilter] = React.useState('');

  return (
    <div style={{ display: 'grid', gridTemplateRows: '1fr auto', gap: '1rem', height: '100%' }}>
      <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#000', overflowY: 'auto', display: 'flex', flexDirection: 'column' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid #222', paddingBottom: '0.5rem' }}>
          <h2 style={{ color: '#00ffff', margin: 0 }}>NEURAL BUS [SPINE COMMS]</h2>
          <input 
            type="text" 
            placeholder="FILTER TELEMETRY..." 
            onChange={(e) => setFilter(e.target.value)}
            style={{ background: 'transparent', border: '1px solid #333', color: '#888', padding: '4px 8px', fontSize: '0.7rem', outline: 'none' }}
          />
        </div>
        <div style={{ flexGrow: 1, overflowY: 'auto', marginTop: '1rem', display: 'flex', flexDirection: 'column-reverse' }}>
          <div>
            {logs.filter(log => !filter || log.message.toLowerCase().includes(filter.toLowerCase()) || log.source.toLowerCase().includes(filter.toLowerCase())).map((log, i) => (
              <div key={i} style={{ borderBottom: '1px solid #111', padding: '0.4rem 0', fontSize: '0.8rem', fontFamily: 'monospace' }}>
                <span style={{ color: '#555', marginRight: '1rem' }}>[{new Date(log.timestamp * 1000).toLocaleTimeString()}]</span>
                <span style={{ color: log.severity === 'ERROR' ? '#ff0000' : '#00ffff', fontWeight: 'bold', marginRight: '0.5rem' }}>{log.source}</span>
                <span style={{ color: '#ccc' }}>{log.message}</span>
              </div>
            ))}
          </div>
        </div>
      </section>
      <div style={{ border: '1px solid #00ff00', background: '#0a0a0a' }}>
        <CommandConsole onCommand={sendCommand} />
      </div>
    </div>
  );
};
