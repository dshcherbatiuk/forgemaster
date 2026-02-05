import { A2UIRenderer } from "./components/A2UIRenderer";
import { taskFormSchema } from "./schemas/taskForm";

function App() {
  const handleAction = (action: string, data: unknown) => {
    console.log("Action:", action, "Data:", data);
  };

  return (
    <div className="min-h-screen bg-gray-900 text-white">
      <header className="bg-gray-800 p-4">
        <h1 className="text-2xl font-bold">ForgeMaster</h1>
      </header>
      <main className="p-4">
        <A2UIRenderer schema={taskFormSchema} onAction={handleAction} />
      </main>
    </div>
  );
}

export default App;
