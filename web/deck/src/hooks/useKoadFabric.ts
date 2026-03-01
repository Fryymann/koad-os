import { useState, useEffect, useRef } from 'react';

export interface SystemStats {
  cpu_usage: number;
  memory_usage: number;
  uptime: number;
  timestamp: number;
}

export interface TelemetryEvent {
  source: string;
  message: string;
  timestamp: number;
  level: number;
}

export function useKoadFabric() {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [logs, setLogs] = useState<TelemetryEvent[]>([]);
  const ws = useRef<WebSocket | null>(null);

  const sendCommand = (cmd: string) => {
    if (ws.current && ws.current.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify({ type: 'COMMAND', payload: cmd }));
      return true;
    }
    return false;
  };

  useEffect(() => {
    const host = window.location.hostname;
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    // The Kernel WebGateway is on port 3000
    const socketUrl = `${protocol}//${host}:3000/ws/fabric`;
    
    const connect = () => {
      console.log("Deck: Connecting to Spine Fabric...");
      const socket = new WebSocket(socketUrl);
      ws.current = socket;

      socket.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          if (data.cpu_usage !== undefined) {
            setStats(data);
          } else if (data.message) {
            setLogs(prev => [data, ...prev].slice(0, 100));
          }
        } catch (e) {
          // Raw message fallback
          setLogs(prev => [{
            source: 'KERNEL',
            message: event.data,
            timestamp: Date.now() / 1000,
            level: 0
          }, ...prev].slice(0, 100));
        }
      };

      socket.onclose = () => {
        console.warn("Deck: Connection lost. Retrying in 5s...");
        setTimeout(connect, 5000);
      };
    };

    connect();
    return () => ws.current?.close();
  }, []);

  return { stats, logs, sendCommand };
}
