import { useMemo } from "react";
import { A2UIRenderer } from "./components/A2UIRenderer";
import type { A2UISchema } from "./components/A2UIRenderer";
import { ConnectionStatus } from "./components/ConnectionStatus";
import { useSchema } from "./hooks/useSchema";
import { useWebSocket } from "./hooks/useWebSocket";
import type { ServerSchema } from "./hooks/useWebSocket";

const WS_URL = import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws";

function mergeSchemas(
  base: A2UISchema,
  server: ServerSchema | undefined,
): A2UISchema {
  if (!server) return base;

  const serverComponents = server.components as A2UISchema["components"];

  // ID-based merge: server components override static ones with the same ID
  const componentMap = new Map(
    base.components.map((c) => [c.id, c]),
  );
  for (const component of serverComponents) {
    componentMap.set(component.id, component);
  }

  return {
    ...base,
    components: [...componentMap.values()],
  };
}

function App() {
  const staticSchema = useSchema("dashboard");
  const { connected, data, serverSchema, sendCommand } = useWebSocket(WS_URL);

  const activeSchema = useMemo(
    () => (staticSchema ? mergeSchemas(staticSchema, serverSchema) : undefined),
    [staticSchema, serverSchema],
  );

  const mergedData = useMemo(() => {
    if (!serverSchema) return data;
    return { ...data, ...serverSchema.data };
  }, [data, serverSchema]);

  const handleAction = (action: { actionName: string; context?: Record<string, unknown> }) => {
    console.log("Action:", action);
    sendCommand(action.actionName, action.context ?? {});
  };

  if (!activeSchema) {
    return <div className="min-h-screen bg-teal-100 text-teal-900 p-4">Loading...</div>;
  }

  return (
    <div className="min-h-screen bg-teal-100 text-teal-900 p-4">
      <A2UIRenderer schema={activeSchema} data={mergedData} onAction={handleAction} />
      <ConnectionStatus connected={connected} />
    </div>
  );
}

export default App;
