import type { Metadata } from "next";
import { TaskDetail } from "@/components/use/TaskDetail";
import { getTask } from "@/lib/api";
import { TaskDetail as TaskDetailType } from "@/lib/types";

export function generateStaticParams() {
  // Return a single placeholder - actual task data is loaded at runtime
  return [{ id: "placeholder" }];
}

export async function generateMetadata({ params }: { params: Promise<{ id: string }> }): Promise<Metadata> {
  const { id } = await params;
  return { title: `Task ${id}` };
}

export default async function TaskDetailPage({ params }: { params: Promise<{ id: string }> }) {
  const { id } = await params;
  let task: TaskDetailType | null = null;
  let error: string | null = null;

  try {
    task = await getTask(id);
  } catch (e: any) {
    error = e.message;
  }

  if (error) return <div className="p-8 text-center text-red-600">{error}</div>;
  if (!task) return <div className="p-8 text-center">Task not found</div>;

  return (
    <div className="container mx-auto py-8 px-4">
      <TaskDetail task={task} />
    </div>
  );
}