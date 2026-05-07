import { TaskStatus } from "@/lib/types";

interface Props {
  status: TaskStatus;
}

const STATUS_CONFIG: Record<TaskStatus, { label: string; className: string }> = {
  submitted: { label: "Submitted", className: "bg-gray-100 text-gray-700" },
  queued: { label: "Queued", className: "bg-blue-100 text-blue-700" },
  processing: { label: "Processing", className: "bg-yellow-100 text-yellow-700" },
  needs_review: { label: "Needs Review", className: "bg-orange-100 text-orange-700" },
  done: { label: "Done", className: "bg-green-100 text-green-700" },
  failed: { label: "Failed", className: "bg-red-100 text-red-700" },
  cancelled: { label: "Cancelled", className: "bg-gray-100 text-gray-500" },
};

export function TaskStatusBadge({ status }: Props) {
  const config = STATUS_CONFIG[status];
  return (
    <span className={`inline-block px-2 py-1 rounded text-xs font-medium ${config.className}`}>
      {config.label}
    </span>
  );
}