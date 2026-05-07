import { Task } from "@/lib/types";
import { TaskStatusBadge } from "./TaskStatusBadge";

interface Props {
  task: Task;
}

export function TaskCard({ task }: Props) {
  return (
    <div className="bg-white rounded-lg shadow p-4 hover:shadow-md transition-shadow">
      <div className="flex items-start justify-between">
        <div>
          <h3 className="font-semibold text-warm-900">{task.title}</h3>
          <p className="text-sm text-warm-500 mt-1">
            Created {new Date(task.createdAt).toLocaleDateString()}
          </p>
        </div>
        <TaskStatusBadge status={task.status} />
      </div>
    </div>
  );
}