import "../styles/connectionstatus.css";

interface Props {
  connected: boolean;
}

export function ConnectionStatus({ connected }: Props) {
  return (
    <div className="connection-status">
      <span className={`connection-dot ${connected ? "connected" : "disconnected"}`} />
      <span className="connection-label">
        {connected ? "Connected" : "Reconnecting..."}
      </span>
    </div>
  );
}
