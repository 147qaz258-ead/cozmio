"use client";

import { useState, useEffect } from "react";
import { TaskDetail } from "@/components/use/TaskDetail";
import { getTask } from "@/lib/api";
import { TaskDetail as TaskDetailType } from "@/lib/types";

interface Props {
  id: string;
}

export default function TaskDetailClient({ id }: Props) {
  const [task, setTask] = useState<TaskDetailType | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!id) return;
    getTask(id)
      .then(setTask)
      .catch(err => setError(err.message))
      .finally(() => setLoading(false));
  }, [id]);

  if (loading) return <div className="p-8 text-center">Loading...</div>;
  if (error) return <div className="p-8 text-center text-red-600">{error}</div>;
  if (!task) return <div className="p-8 text-center">Task not found</div>;

  return (
    <div className="container mx-auto py-8 px-4">
      <TaskDetail task={task} />
    </div>
  );
}