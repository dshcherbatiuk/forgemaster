import { useState } from "react";
import { A2UIRenderer } from "./components/A2UIRenderer";
import { useSchema } from "./hooks/useSchema";
import { getAvailableSchemas } from "./schemas/schemaLoader";

interface PanelProps {
  schemaName: string;
  data?: Record<string, unknown>;
  onAction?: (action: unknown) => void;
}

function DynamicPanel({ schemaName, data, onAction }: PanelProps) {
  const schema = useSchema(schemaName);

  if (!schema) return <div className="p-4 text-red-400">Schema not found: {schemaName}</div>;

  return <A2UIRenderer schema={schema} data={data} onAction={onAction} />;
}

function App() {
  const availableSchemas = getAvailableSchemas();
  const [activePanels, setActivePanels] = useState<string[]>(["taskForm"]);

  const handleAction = (action: unknown) => {
    console.log("Action:", action);
  };

  const togglePanel = (name: string) => {
    setActivePanels((panels) =>
      panels.includes(name) ? panels.filter((p) => p !== name) : [...panels, name]
    );
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="bg-gray-800 p-4">
        <h1 className="text-2xl font-bold">ForgeMaster</h1>
      </header>

      <nav className="bg-gray-700 p-2 flex gap-2 flex-wrap">
        {availableSchemas.map((name) => (
          <button
            key={name}
            onClick={() => togglePanel(name)}
            className={`px-3 py-1 rounded text-sm ${
              activePanels.includes(name)
                ? "bg-blue-600 text-white"
                : "bg-gray-600 text-gray-300 hover:bg-gray-500"
            }`}
          >
            {name}
          </button>
        ))}
      </nav>

      <main className="p-4 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {activePanels.map((name) => (
          <DynamicPanel key={name} schemaName={name} onAction={handleAction} />
        ))}
      </main>
    </div>
  );
}

export default App;
