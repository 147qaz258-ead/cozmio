export interface User {
  id: string;
  email: string;
  name?: string;
  role: "user" | "admin";
}

export type TaskStatus = "submitted" | "queued" | "processing" | "needs_review" | "done" | "failed" | "cancelled";

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  createdAt: string;
  updatedAt: string;
  resultSummary?: string;
  errorMessage?: string;
  shareToken?: string;
}

export interface TaskDetail extends Task {
  prompt: string;
  sourceUrl?: string;
  sourceType?: string;
  events: TaskEvent[];
}

export interface TaskEvent {
  id: string;
  eventType: string;
  message?: string;
  createdAt: string;
}

export interface ApiResponse<T> {
  ok: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
  };
}

export interface PaginatedResponse<T> extends ApiResponse<T[]> {
  pagination: {
    page: number;
    pageSize: number;
    total: number;
  };
}