import { User, Task, TaskDetail, TaskEvent, TaskStatus, ApiResponse, PaginatedResponse } from "./types";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "/api";

// MOCK_MODE: Demo mode with localStorage-based auth when no backend is available
const MOCK_MODE = true; // Set to false when real backend is deployed

function getCookie(name: string): string | undefined {
  if (typeof document === "undefined") return undefined;
  const value = `; ${document.cookie}`;
  const parts = value.split(`; ${name}=`);
  if (parts.length === 2) return parts.pop()?.split(";").shift();
}

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const cookie = getCookie("session_token");
  const headers: Record<string, string> = {
    "Content-Type": "application/json",
    ...(cookie ? { Authorization: `Bearer ${cookie}` } : {}),
  };

  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers,
    credentials: "include",
  });

  const data = await res.json();
  if (!res.ok || !data.ok) {
    throw new Error(data.error?.message || "Request failed");
  }
  return data.data;
}

// Mock user for demo mode
const mockUser: User = {
  id: "mock-user-1",
  email: "",
  name: "Demo User",
  role: "user",
};

// Tasks
const mockTasks: Task[] = [
  {
    id: "task-1",
    title: "分析竞品页面设计",
    status: "done" as TaskStatus,
    createdAt: new Date(Date.now() - 86400000).toISOString(),
    updatedAt: new Date(Date.now() - 3600000).toISOString(),
    resultSummary: "完成了对三个竞品页面的分析...",
  },
  {
    id: "task-2",
    title: "生成API文档",
    status: "processing" as TaskStatus,
    createdAt: new Date(Date.now() - 3600000).toISOString(),
    updatedAt: new Date(Date.now() - 1800000).toISOString(),
  },
];

const mockTaskDetail: TaskDetail = {
  id: "task-1",
  title: "分析竞品页面设计",
  status: "done" as TaskStatus,
  createdAt: new Date(Date.now() - 86400000).toISOString(),
  updatedAt: new Date(Date.now() - 3600000).toISOString(),
  resultSummary: "完成了对三个竞品页面的分析...",
  prompt: "请分析竞品的产品页面设计风格、配色方案和布局特点。",
  events: [
    { id: "e1", eventType: "created", message: "任务已创建", createdAt: new Date(Date.now() - 86400000).toISOString() },
    { id: "e2", eventType: "started", message: "开始处理", createdAt: new Date(Date.now() - 7200000).toISOString() },
    { id: "e3", eventType: "done", message: "任务完成", createdAt: new Date(Date.now() - 3600000).toISOString() },
  ],
};

// Demo OTP code - in real app this would be sent via email
const DEMO_OTP = "123456";

function saveMockSession(email: string) {
  if (typeof localStorage === "undefined") return;
  localStorage.setItem("mock_session", JSON.stringify({
    email,
    user: { ...mockUser, email },
    logged_in_at: new Date().toISOString(),
  }));
}

function getMockSession(): { email: string; user: User } | null {
  if (typeof localStorage === "undefined") return null;
  const data = localStorage.getItem("mock_session");
  return data ? JSON.parse(data) : null;
}

function clearMockSession() {
  if (typeof localStorage === "undefined") return;
  localStorage.removeItem("mock_session");
}

// Auth - with mock fallback
export async function sendCode(email: string) {
  if (MOCK_MODE) {
    // Simulate network delay
    await new Promise(resolve => setTimeout(resolve, 800));
    // Store email for verify step
    if (typeof localStorage !== "undefined") {
      localStorage.setItem("mock_pending_email", email);
    }
    return { message: "验证码已发送（演示模式：使用 123456）" };
  }
  return request<{ message: string }>("/auth/send-code", {
    method: "POST",
    body: JSON.stringify({ email, type: "otp" }),
  });
}

export async function verifyCode(email: string, token: string) {
  if (MOCK_MODE) {
    await new Promise(resolve => setTimeout(resolve, 600));
    const pendingEmail = typeof localStorage !== "undefined" ? localStorage.getItem("mock_pending_email") : null;
    const useEmail = email || pendingEmail;
    if (!useEmail) {
      throw new Error("请先输入邮箱");
    }
    if (token !== DEMO_OTP) {
      throw new Error("验证码错误（演示模式：使用 123456）");
    }
    const user = { ...mockUser, email: useEmail };
    saveMockSession(useEmail);
    return { user, session_token: "mock-token-" + Date.now(), expires_at: new Date(Date.now() + 86400000).toISOString() };
  }
  return request<{ user: User; session_token: string; expires_at: string }>("/auth/verify", {
    method: "POST",
    body: JSON.stringify({ email, token }),
  });
}

export async function logout() {
  if (MOCK_MODE) {
    clearMockSession();
    return { message: "已退出" };
  }
  return request<{ message: string }>("/auth/logout", { method: "POST" });
}

export async function getMe() {
  if (MOCK_MODE) {
    const session = getMockSession();
    if (!session) {
      // Return demo user directly without login
      return { ...mockUser, email: "demo@cozmio.app" };
    }
    return session.user;
  }
  return request<User>("/auth/me");
}

// Tasks - with mock fallback
export async function getTasks(params?: { page?: number; pageSize?: number; status?: string }) {
  if (MOCK_MODE) {
    await new Promise(resolve => setTimeout(resolve, 300));
    let tasks = [...mockTasks];
    if (params?.status) {
      tasks = tasks.filter(t => t.status === params.status);
    }
    return tasks;
  }
  const searchParams = new URLSearchParams();
  if (params?.page) searchParams.set("page", String(params.page));
  if (params?.pageSize) searchParams.set("pageSize", String(params.pageSize));
  if (params?.status) searchParams.set("status", params.status);
  const query = searchParams.toString();
  return request<Task[]>(`/tasks${query ? `?${query}` : ""}`);
}

export async function getTask(id: string) {
  if (MOCK_MODE) {
    await new Promise(resolve => setTimeout(resolve, 200));
    if (id === "task-1") {
      return { ...mockTaskDetail };
    }
    if (id === "task-2") {
      const task2Detail: TaskDetail = {
        id: "task-2",
        title: "生成API文档",
        status: "processing" as TaskStatus,
        createdAt: new Date(Date.now() - 3600000).toISOString(),
        updatedAt: new Date(Date.now() - 1800000).toISOString(),
        prompt: "为新的用户认证模块生成详细的API接口文档。",
        events: [
          { id: "e1", eventType: "created", message: "任务已创建", createdAt: new Date(Date.now() - 3600000).toISOString() },
          { id: "e2", eventType: "started", message: "开始处理", createdAt: new Date(Date.now() - 1800000).toISOString() },
        ],
      };
      return task2Detail;
    }
    throw new Error("任务不存在");
  }
  return request<TaskDetail>(`/tasks/${id}`);
}

export async function createTask(data: { title: string; prompt: string; sourceUrl?: string }) {
  if (MOCK_MODE) {
    await new Promise(resolve => setTimeout(resolve, 500));
    const newTask: Task = {
      id: "task-" + Date.now(),
      title: data.title,
      status: "submitted" as TaskStatus,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString(),
    };
    mockTasks.unshift(newTask);
    return { id: newTask.id, status: newTask.status };
  }
  return request<{ id: string; status: string }>("/tasks", {
    method: "POST",
    body: JSON.stringify(data),
  });
}
