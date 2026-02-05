import { A2UIRenderer } from "./components/A2UIRenderer";
import { useSchema } from "./hooks/useSchema";

function App() {
  const schema = useSchema("dashboard");

  const handleAction = (action: unknown) => {
    console.log("Action:", action);
  };

  if (!schema) {
    return <div className="min-h-screen bg-gray-900 text-white p-4">Loading...</div>;
  }

  return (
    <div className="min-h-screen bg-gray-900 text-white p-4">
      <A2UIRenderer schema={schema} onAction={handleAction} />
    </div>
  );
}

export default App;
