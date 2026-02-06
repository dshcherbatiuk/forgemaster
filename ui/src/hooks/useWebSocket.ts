import { useCallback, useEffect, useRef, useState } from "react";

const MAX_RECONNECT_DELAY_MS = 30_000;
const INITIAL_RECONNECT_DELAY_MS = 3_000;

export interface ServerSchema {
  root: string;
  components: unknown[];
  data: Record<string, unknown>;
}

interface UseWebSocketResult {
  connected: boolean;
  data: Record<string, unknown> | undefined;
  serverSchema: ServerSchema | undefined;
  sendCommand: (type: string, fields: Record<string, unknown>) => void;
}

export function useWebSocket(url: string): UseWebSocketResult {
  const [connected, setConnected] = useState(false);
  const [data, setData] = useState<Record<string, unknown> | undefined>();
  const [serverSchema, setServerSchema] = useState<ServerSchema | undefined>();
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectDelayRef = useRef(INITIAL_RECONNECT_DELAY_MS);
  const reconnectTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const connect = useCallback(() => {
    const socket = new WebSocket(url);
    wsRef.current = socket;

    socket.onopen = () => {
      console.log("[WS] Connected");
      setConnected(true);
      reconnectDelayRef.current = INITIAL_RECONNECT_DELAY_MS;
    };

    socket.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        console.log("[WS] Received:", message.type);

        if (message.type === "data") {
          setData(message.data);
        } else if (message.type === "schema") {
          setServerSchema({
            root: message.root,
            components: message.components,
            data: message.data,
          });
        }
      } catch {
        console.warn("[WS] Failed to parse message");
      }
    };

    socket.onclose = () => {
      console.log("[WS] Disconnected");
      setConnected(false);
      wsRef.current = null;

      const delay = reconnectDelayRef.current;
      reconnectDelayRef.current = Math.min(delay * 2, MAX_RECONNECT_DELAY_MS);
      reconnectTimerRef.current = setTimeout(connect, delay);
    };

    socket.onerror = () => {
      socket.close();
    };
  }, [url]);

  useEffect(() => {
    connect();

    return () => {
      if (reconnectTimerRef.current) {
        clearTimeout(reconnectTimerRef.current);
      }
      wsRef.current?.close();
    };
  }, [connect]);

  const sendCommand = useCallback(
    (type: string, fields: Record<string, unknown>) => {
      const socket = wsRef.current;
      if (!socket || socket.readyState !== WebSocket.OPEN) {
        console.warn("[WS] Cannot send command: not connected");
        return;
      }
      socket.send(JSON.stringify({ type, ...fields }));
    },
    [],
  );

  return { connected, data, serverSchema, sendCommand };
}
