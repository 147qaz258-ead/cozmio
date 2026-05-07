"use client";

import { useState, useEffect } from "react";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

const STATUS_TABS = ["all", "pending", "done", "failed"];

export default function TasksPage() {
  const [tasks, setTasks] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [status, setStatus] = useState("all");
  const [search, setSearch] = useState("");
  const [selectedTask, setSelectedTask] = useState<any | null>(null);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    setLoading(true);
    const params = new URLSearchParams({ page: String(page), pageSize: "20" });
    if (status !== "all") params.set("status", status);
    if (search) params.set("search", search);

    fetch(`${API_BASE}/api/admin/tasks?${params}`, { credentials: "include" })
      .then(r => r.json())
      .then(data => {
        setTasks(data.data || []);
        setTotal(data.pagination?.total || 0);
      })
      .finally(() => setLoading(false));
  }, [status, search, page]);

  const handleUpdateTask = async (id: string, payload: { status?: string; result_summary?: string; internal_note?: string }) => {
    await fetch(`${API_BASE}/api/admin/tasks/${id}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify(payload),
    });
    setSelectedTask(null);
    setPage(1);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold text-[#151515]">Tasks</h1>
        <div className="flex items-center gap-4">
          <input
            type="text"
            placeholder="Search..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            className="coz-input"
          />
        </div>
      </div>

      <div className="flex gap-2 mb-6">
        {STATUS_TABS.map(tab => (
          <button
            key={tab}
            onClick={() => { setStatus(tab); setPage(1); }}
            className={`px-4 py-2 rounded-xl text-sm font-bold ${
              status === tab ? "bg-[#151515] text-white" : "bg-white text-[#625b54]"
            }`}
          >
            {tab.charAt(0).toUpperCase() + tab.slice(1)}
          </button>
        ))}
      </div>

      <div className="bg-white rounded-2xl shadow overflow-hidden">
        <table className="w-full">
          <thead className="bg-[#faf8f5]">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Title</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Email</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Status</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Created</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr><td colSpan={5} className="p-8 text-center">Loading...</td></tr>
            ) : tasks.length === 0 ? (
              <tr><td colSpan={5} className="p-8 text-center text-[#625b54]">No tasks found</td></tr>
            ) : tasks.map(task => (
              <tr key={task.id} className="border-t border-black/4">
                <td className="px-6 py-4 font-medium text-[#151515]">{task.title}</td>
                <td className="px-6 py-4 text-[#625b54]">{task.email}</td>
                <td className="px-6 py-4">
                  <span className={`text-xs px-2 py-1 rounded ${
                    task.status === "done" ? "bg-green-100" :
                    task.status === "failed" ? "bg-red-100" : "bg-gray-100"
                  }`}>{task.status}</span>
                </td>
                <td className="px-6 py-4 text-[#625b54]">{new Date(task.createdAt).toLocaleDateString()}</td>
                <td className="px-6 py-4">
                  <button onClick={() => setSelectedTask(task)} className="text-sm font-bold text-[#151515] hover:underline">
                    View
                  </button>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div className="flex justify-center gap-2 mt-6">
        <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page === 1} className="coz-btn-dark">
          Previous
        </button>
        <span className="px-4 py-2 text-[#625b54]">Page {page} of {Math.ceil(total / 20) || 1}</span>
        <button onClick={() => setPage(p => p + 1)} disabled={page * 20 >= total} className="coz-btn-dark">
          Next
        </button>
      </div>

      {/* Detail Modal */}
      {selectedTask && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-3xl p-8 w-full max-w-lg max-h-[90vh] overflow-auto">
            <h2 className="text-2xl font-bold text-[#151515] mb-4">Task Details</h2>
            <div className="space-y-4">
              <div><label className="text-sm text-[#625b54]">Title</label><div className="font-medium">{selectedTask.title}</div></div>
              <div><label className="text-sm text-[#625b54]">Email</label><div className="font-medium">{selectedTask.email}</div></div>
              <div><label className="text-sm text-[#625b54]">Description</label><div className="font-medium">{selectedTask.description}</div></div>
              <div><label className="text-sm text-[#625b54]">Status</label>
                <select
                  id="task-status"
                  defaultValue={selectedTask.status}
                  className="coz-input w-full mt-1"
                  onChange={e => selectedTask.status = e.target.value}
                >
                  <option value="pending">pending</option>
                  <option value="done">done</option>
                  <option value="failed">failed</option>
                </select>
              </div>
              <div><label className="text-sm text-[#625b54]">Result Summary</label>
                <textarea
                  id="task-result"
                  defaultValue={selectedTask.result_summary || ""}
                  className="coz-input w-full mt-1"
                  rows={3}
                  onChange={e => selectedTask.result_summary = e.target.value}
                />
              </div>
              <div><label className="text-sm text-[#625b54]">Internal Note</label>
                <textarea
                  id="task-note"
                  defaultValue={selectedTask.internal_note || ""}
                  className="coz-input w-full mt-1"
                  rows={3}
                  onChange={e => selectedTask.internal_note = e.target.value}
                />
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button
                onClick={() => handleUpdateTask(selectedTask.id, {
                  status: (document.getElementById("task-status") as HTMLSelectElement).value,
                  result_summary: (document.getElementById("task-result") as HTMLTextAreaElement).value,
                  internal_note: (document.getElementById("task-note") as HTMLTextAreaElement).value,
                })}
                className="coz-btn-dark flex-1"
              >
                Save
              </button>
              <button onClick={() => setSelectedTask(null)} className="coz-btn-light flex-1">
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}