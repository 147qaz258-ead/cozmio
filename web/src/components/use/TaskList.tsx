"use client";

import Link from "next/link";
import { Task } from "@/lib/types";
import { TaskCard } from "./TaskCard";

interface Props {
  tasks: Task[];
}

export function TaskList({ tasks }: Props) {
  if (tasks.length === 0) {
    return (
      <div className="text-center py-12 text-warm-500">
        No tasks yet. Create your first task to get started.
      </div>
    );
  }

  return (
    <div className="grid gap-4">
      {tasks.map(task => (
        <Link key={task.id} href={`/use/tasks/${task.id}`}>
          <TaskCard task={task} />
        </Link>
      ))}
    </div>
  );
}