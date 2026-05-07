"use client";

import { useState, useEffect } from "react";
import { getTasks } from "@/lib/api";
import { Task } from "@/lib/types";
import Link from "next/link";

const DEMO_USER = {
  id: "demo-user",
  email: "demo@cozmio.app",
  name: "Demo User",
  role: "user" as const,
};

export function Workspace() {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getTasks()
      .then(setTasks)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const handleLogout = () => {
    window.location.reload();
  };

  return (
    <div>
      {/* Header */}
      <header className="bg-white border-b border-warm-200 px-6 py-4">
        <div className="container mx-auto flex items-center justify-between">
          <Link href="/use" className="text-xl font-bold text-warm-900">Cozmio</Link>
          <div className="flex items-center gap-4">
            <span className="text-warm-600">{DEMO_USER.email}</span>
            <button
              onClick={handleLogout}
              className="px-3 py-1 text-sm text-warm-600 hover:text-warm-900"
            >
              退出
            </button>
          </div>
        </div>
      </header>

      {/* Nav */}
      <nav className="bg-white border-b border-warm-200 px-6">
        <div className="container mx-auto">
          <div className="flex gap-6 py-3">
            <Link href="/use/tasks" className="text-warm-700 hover:text-primary-600">任务列表</Link>
          </div>
        </div>
      </nav>

      {/* Main Content */}
      <main className="container mx-auto py-8 px-4">
        <h1 className="text-3xl font-bold text-warm-900 mb-4">欢迎使用 Cozmio</h1>
        <p className="text-warm-600 mb-6">管理您的任务，跟踪进度</p>

        {loading ? (
          <p className="text-warm-500">加载中...</p>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            <Link
              href="/use/tasks"
              className="p-6 bg-white rounded-lg shadow hover:shadow-lg transition-shadow"
            >
              <h2 className="text-lg font-semibold text-warm-900">我的任务</h2>
              <p className="text-warm-600 mt-1">查看和管理您的任务 ({tasks.length} 个任务)</p>
            </Link>
            <Link
              href="/request"
              className="p-6 bg-white rounded-lg shadow hover:shadow-lg transition-shadow"
            >
              <h2 className="text-lg font-semibold text-warm-900">提交新任务</h2>
              <p className="text-warm-600 mt-1">创建一个新任务</p>
            </Link>
          </div>
        )}
      </main>
    </div>
  );
}