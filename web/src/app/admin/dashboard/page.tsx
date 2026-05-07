"use client";

import { useState, useEffect } from "react";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

interface Stats {
  userCount: number;
  applicationCount: number;
  taskCount: number;
  downloadCount: number;
}

export default function DashboardPage() {
  const [stats, setStats] = useState<Stats>({ userCount: 0, applicationCount: 0, taskCount: 0, downloadCount: 0 });
  const [recentApps, setRecentApps] = useState<any[]>([]);
  const [recentTasks, setRecentTasks] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      fetch(`${API_BASE}/api/admin/users?pageSize=1`).then(r => r.json()),
      fetch(`${API_BASE}/api/admin/applications?pageSize=1`).then(r => r.json()),
      fetch(`${API_BASE}/api/admin/tasks?pageSize=1`).then(r => r.json()),
      fetch(`${API_BASE}/api/admin/downloads?pageSize=1`).then(r => r.json()),
      fetch(`${API_BASE}/api/admin/applications?pageSize=5`).then(r => r.json()),
      fetch(`${API_BASE}/api/admin/tasks?pageSize=5`).then(r => r.json()),
    ]).then(([users, apps, tasks, downloads, recentApps, recentTasks]) => {
      setStats({
        userCount: users.pagination?.total || 0,
        applicationCount: apps.pagination?.total || 0,
        taskCount: tasks.pagination?.total || 0,
        downloadCount: downloads.pagination?.total || 0,
      });
      setRecentApps(recentApps.data || []);
      setRecentTasks(recentTasks.data || []);
    }).finally(() => setLoading(false));
  }, []);

  if (loading) return <div>Loading...</div>;

  return (
    <div>
      <h1 className="text-3xl font-bold text-[#151515] mb-8">Dashboard</h1>

      <div className="grid grid-cols-4 gap-6 mb-8">
        {[
          { label: "Users", value: stats.userCount, icon: "👤" },
          { label: "Applications", value: stats.applicationCount, icon: "📋" },
          { label: "Tasks", value: stats.taskCount, icon: "✓" },
          { label: "Downloads", value: stats.downloadCount, icon: "⬇" },
        ].map(stat => (
          <div key={stat.label} className="bg-white rounded-2xl p-6 shadow-sm">
            <div className="text-4xl mb-2">{stat.icon}</div>
            <div className="text-3xl font-bold text-[#151515]">{stat.value}</div>
            <div className="text-sm text-[#625b54]">{stat.label}</div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-2 gap-8">
        <div className="bg-white rounded-2xl p-6 shadow-sm">
          <h2 className="text-lg font-bold text-[#151515] mb-4">Recent Applications</h2>
          <div className="space-y-3">
            {recentApps.map(app => (
              <div key={app.id} className="flex items-center justify-between py-2 border-b border-black/4">
                <div>
                  <div className="font-medium text-[#151515]">{app.name}</div>
                  <div className="text-sm text-[#625b54]">{app.email}</div>
                </div>
                <span className="text-xs px-2 py-1 rounded bg-gray-100">{app.status}</span>
              </div>
            ))}
            {recentApps.length === 0 && <p className="text-[#625b54]">No applications yet</p>}
          </div>
        </div>

        <div className="bg-white rounded-2xl p-6 shadow-sm">
          <h2 className="text-lg font-bold text-[#151515] mb-4">Recent Tasks</h2>
          <div className="space-y-3">
            {recentTasks.map(task => (
              <div key={task.id} className="flex items-center justify-between py-2 border-b border-black/4">
                <div>
                  <div className="font-medium text-[#151515]">{task.title}</div>
                  <div className="text-sm text-[#625b54]">{task.email}</div>
                </div>
                <span className={`text-xs px-2 py-1 rounded ${
                  task.status === "done" ? "bg-green-100" : task.status === "failed" ? "bg-red-100" : "bg-gray-100"
                }`}>{task.status}</span>
              </div>
            ))}
            {recentTasks.length === 0 && <p className="text-[#625b54]">No tasks yet</p>}
          </div>
        </div>
      </div>
    </div>
  );
}