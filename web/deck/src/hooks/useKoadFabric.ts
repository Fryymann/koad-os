import { useState, useEffect, useRef } from 'react';

export interface SystemStats {
  cpu_usage: number;
  memory_usage: number;
  uptime: number;
  timestamp: number;
  skill_count: number;
  active_tasks: number;
  history?: SystemStats[];
}

export interface TelemetryEvent {
  source: string;
  message: string;
  timestamp: number;
  level: number;
}

export interface AgentSession {
  session_id: string;
  identity: {
    name: string;
    rank: string;
    permissions: string[];
  };
  environment: string;
  context: {
    project_name: string;
    root_path: string;
  };
  last_heartbeat: string;
  metadata: Record<string, string>;
}

export interface ProjectIssue {
  id: string;
  title: string;
  status: string;
  number?: number;
  target_version?: string;
}

export interface ProjectMapItem {
  id: number;
  name: string;
  path: string;
  branch: string;
  health: string;
}

export function useKoadFabric() {
  const [stats, setStats] = useState<SystemStats | null>(null);
  const [logs, setLogs] = useState<TelemetryEvent[]>([]);
  const [agents, setAgents] = useState<AgentSession[]>([]);
  const [issues, setIssues] = useState<ProjectIssue[]>([]);
  const [projects, setProjects] = useState<ProjectMapItem[]>([]);
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
          if (data.type === 'SYSTEM_SYNC') {
            setAgents(data.payload.agents || []);
            setIssues(data.payload.issues || []);
            setProjects(data.payload.projects || []);
          } else if (data.type === 'SESSION_UPDATE') {
            const updatedSession = data.payload;
            setAgents(prev => {
              const exists = prev.find(a => a.session_id === updatedSession.session_id);
              if (exists) {
                return prev.map(a => a.session_id === updatedSession.session_id ? updatedSession : a);
              } else {
                return [...prev, updatedSession];
              }
            });
          } else if (data.cpu_usage !== undefined) {
            setStats(prev => ({
              ...data,
              history: prev ? [...(prev.history || []), data].slice(-20) : [data]
            }));
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

  return { stats, logs, agents, issues, projects, sendCommand };
}
