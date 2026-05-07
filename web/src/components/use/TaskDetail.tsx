import { TaskDetail as TaskDetailType } from "@/lib/types";
import { TaskStatusBadge } from "./TaskStatusBadge";

interface Props {
  task: TaskDetailType;
}

export function TaskDetail({ task }: Props) {
  return (
    <div className="bg-white rounded-lg shadow">
      <div className="p-6 border-b border-warm-200">
        <div className="flex items-start justify-between">
          <h1 className="text-2xl font-bold text-warm-900">{task.title}</h1>
          <TaskStatusBadge status={task.status} />
        </div>
        <p className="text-warm-500 mt-2">Created {new Date(task.createdAt).toLocaleString()}</p>
      </div>

      <div className="p-6 space-y-6">
        <div>
          <h2 className="text-sm font-semibold text-warm-700 uppercase mb-2">Prompt</h2>
          <p className="text-warm-800 whitespace-pre-wrap">{task.prompt}</p>
        </div>

        {task.sourceUrl && (
          <div>
            <h2 className="text-sm font-semibold text-warm-700 uppercase mb-2">Source</h2>
            <a href={task.sourceUrl} target="_blank" rel="noopener noreferrer" className="text-primary-600 hover:underline">
              {task.sourceUrl}
            </a>
          </div>
        )}

        {task.resultSummary && (
          <div>
            <h2 className="text-sm font-semibold text-warm-700 uppercase mb-2">Result</h2>
            <p className="text-warm-800 whitespace-pre-wrap">{task.resultSummary}</p>
          </div>
        )}

        {task.errorMessage && (
          <div>
            <h2 className="text-sm font-semibold text-warm-700 uppercase mb-2">Error</h2>
            <p className="text-red-600 whitespace-pre-wrap">{task.errorMessage}</p>
          </div>
        )}

        {task.events && task.events.length > 0 && (
          <div>
            <h2 className="text-sm font-semibold text-warm-700 uppercase mb-2">Timeline</h2>
            <div className="space-y-2">
              {task.events.map(event => (
                <div key={event.id} className="flex gap-3 text-sm">
                  <span className="text-warm-400">{new Date(event.createdAt).toLocaleString()}</span>
                  <span className="text-warm-600">{event.message || event.eventType}</span>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}