import { A2UIRenderer } from "./components/A2UIRenderer";
import { ConnectionStatus } from "./components/ConnectionStatus";
import { useSchema } from "./hooks/useSchema";
import { useWebSocket } from "./hooks/useWebSocket";

const WS_URL = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws";

function App() {
  const schema = useSchema("dashboard");
  const { connected, data, sendCommand } = useWebSocket(WS_URL);

  const handleAction = (action: { actionName: string; context?: Record<string, unknown> }) => {
    console.log("Action:", action);
    sendCommand(action.actionName, action.context ?? {});
  };

  if (!schema) {
    return <div className="min-h-screen bg-teal-100 text-teal-900 p-4">Loading...</div>;
  }

  return (
    <div className="min-h-screen bg-teal-100 text-teal-900 p-4">
      <A2UIRenderer schema={schema} data={data} onAction={handleAction} />
      <ConnectionStatus connected={connected} />
    </div>
  );
}

export default App;
