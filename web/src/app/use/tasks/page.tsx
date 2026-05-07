"use client";

import { useState, useEffect } from "react";
import { TaskList } from "@/components/use/TaskList";
import { TaskSubmitForm } from "@/components/use/TaskSubmitForm";
import { getTasks } from "@/lib/api";
import { Task } from "@/lib/types";

// Opt out of static generation for this dynamic route - handles it as client-side route
export const dynamicParams = true;

export default function TasksPage() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [showSubmit, setShowSubmit] = useState(false);
  const [refreshKey, setRefreshKey] = useState(0);

  useEffect(() => {
    getTasks({ pageSize: 50 }).then(setTasks).catch(console.error);
  }, [refreshKey]);

  const handleTaskCreated = () => {
    setShowSubmit(false);
    setRefreshKey(k => k + 1);
  };

  return (
    <div className="container mx-auto py-8 px-4">
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-2xl font-bold text-warm-900">My Tasks</h1>
        <button
          onClick={() => setShowSubmit(true)}
          className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 transition-colors"
        >
          + New Task
        </button>
      </div>

      {showSubmit && (
        <div className="mb-6">
          <TaskSubmitForm onSuccess={handleTaskCreated} onCancel={() => setShowSubmit(false)} />
        </div>
      )}

      <TaskList tasks={tasks} />
    </div>
  );
}