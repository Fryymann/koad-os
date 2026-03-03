import React, { useState } from 'react';
import { useKoadFabric } from './hooks/useKoadFabric';
import { EngineRoom } from './components/hud/EngineRoom';
import { Bridge } from './components/hud/Bridge';
import { CrewQuarters } from './components/hud/CrewQuarters';
import { Engineering } from './components/hud/Engineering';

type HUDTab = 'BRIDGE' | 'CREW' | 'ENGINEERING' | 'ARCHIVES';

const App: React.FC = () => {
  const { stats, logs, agents, issues, projects, sendCommand } = useKoadFabric();
  const [activeTab, setActiveTab] = useState<HUDTab>('BRIDGE');

  const renderTab = () => {
    switch (activeTab) {
      case 'BRIDGE':
        return <Bridge projects={projects} issues={issues} />;
      case 'CREW':
        return <CrewQuarters agents={agents} />;
      case 'ENGINEERING':
        return <Engineering logs={logs} sendCommand={sendCommand} />;
      case 'ARCHIVES':
        return (
          <section style={{ border: '1px solid #00ff00', padding: '1.5rem', background: '#0a0a0a', height: '100%' }}>
            <h2 style={{ color: '#00ffff', borderBottom: '1px solid #222' }}>THE MIND [ARCHIVES]</h2>
            <div style={{ marginTop: '2rem', textAlign: 'center', color: '#444' }}>
              <p style={{ fontSize: '1.2rem', fontStyle: 'italic' }}>Deep scanning cognitive database...</p>
              <div style={{ marginTop: '1rem', color: '#222' }}>[VECTOR EMBEDDING ENGINE OFFLINE]</div>
            </div>
          </section>
        );
      default:
        return null;
    }
  };

  return (
    <div style={{
      backgroundColor: '#050505',
      color: '#00ff00',
      height: '100vh',
      display: 'grid',
      gridTemplateRows: 'auto 1fr',
      fontFamily: 'monospace',
      overflow: 'hidden'
    }}>
      {/* Admiral's Header */}
      <header style={{ 
        padding: '1rem 2rem', 
        borderBottom: '2px solid #00ff00', 
        display: 'flex', 
        justifyContent: 'space-between', 
        alignItems: 'center',
        background: '#000'
      }}>
        <div>
          <h1 style={{ margin: 0, letterSpacing: '4px', fontSize: '1.5rem' }}>KOADOS // CYBERSHIP HUD</h1>
          <div style={{ fontSize: '0.7rem', color: '#008800' }}>AUTHORIZED PERSONNEL ONLY: ADMIRAL DOOD</div>
        </div>
        
        <nav style={{ display: 'flex', gap: '1rem' }}>
          {(['BRIDGE', 'CREW', 'ENGINEERING', 'ARCHIVES'] as HUDTab[]).map(tab => (
            <button
              key={tab}
              onClick={() => setActiveTab(tab)}
              style={{
                background: activeTab === tab ? '#00ff00' : 'transparent',
                color: activeTab === tab ? '#000' : '#00ff00',
                border: '1px solid #00ff00',
                padding: '0.5rem 1.5rem',
                cursor: 'pointer',
                fontWeight: 'bold',
                transition: 'all 0.2s'
              }}
            >
              {tab}
            </button>
          ))}
        </nav>
      </header>

      {/* Admiral's Grid */}
      <main style={{ 
        display: 'grid', 
        gridTemplateColumns: '350px 1fr', 
        gap: '1rem', 
        padding: '1rem',
        height: 'calc(100vh - 80px)'
      }}>
        {/* Persistent Engine Panel (Always Visible) */}
        <EngineRoom stats={stats} />

        {/* Dynamic Tab Viewport */}
        <div style={{ overflow: 'hidden' }}>
          {renderTab()}
        </div>
      </main>
    </div>
  );
};

export default App;
