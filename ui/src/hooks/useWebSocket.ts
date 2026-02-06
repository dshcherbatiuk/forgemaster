import { useCallback, useEffect, useRef, useState } from "react";

const MAX_RECONNECT_DELAY_MS = 30_000;
const INITIAL_RECONNECT_DELAY_MS = 3_000;

interface UseWebSocketResult {
  connected: boolean;
  data: Record<string, unknown> | undefined;
}

export function useWebSocket(url: string): UseWebSocketResult {
  const [connected, setConnected] = useState(false);
  const [data, setData] = useState<Record<string, unknown> | undefined>();
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

  return { connected, data };
}
