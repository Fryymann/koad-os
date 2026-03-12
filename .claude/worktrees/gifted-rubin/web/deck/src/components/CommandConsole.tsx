import React, { useState } from 'react';

interface CommandConsoleProps {
  onSend: (cmd: string) => boolean;
}

export const CommandConsole: React.FC<CommandConsoleProps> = ({ onSend }) => {
  const [input, setInput] = useState('');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (input.trim()) {
      const success = onSend(input);
      if (success) {
        setInput('');
      } else {
        alert('Spine connection offline. Command failed.');
      }
    }
  };

  return (
    <div style={{ marginTop: 'auto', borderTop: '1px solid #333', paddingTop: '1rem' }}>
      <form onSubmit={handleSubmit} style={{ display: 'flex', gap: '0.5rem' }}>
        <span style={{ color: '#00ff00' }}>{'>'}</span>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          placeholder="ENTER COMMAND..."
          style={{
            flexGrow: 1,
            background: 'transparent',
            border: 'none',
            color: '#00ff00',
            outline: 'none',
            fontFamily: 'monospace',
            fontSize: '1rem'
          }}
          autoFocus
        />
      </form>
    </div>
  );
};
