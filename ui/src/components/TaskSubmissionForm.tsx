import { useState } from "react";
import "../styles/tasksubmissionform.css";

interface Props {
  onSubmit: (description: string) => void;
}

export function TaskSubmissionForm({ onSubmit }: Props) {
  const [description, setDescription] = useState("");

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (description.trim()) {
      onSubmit(description);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="task-form">
      <h2>Create New Task</h2>
      <textarea
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        placeholder="Describe what you want to build..."
      />
      <div className="button-container">
        <button type="submit" disabled={!description.trim()}>
          Submit Task
        </button>
      </div>
    </form>
  );
}
