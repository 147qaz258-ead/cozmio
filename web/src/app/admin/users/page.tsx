"use client";

import { useState, useEffect } from "react";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

export default function UsersPage() {
  const [users, setUsers] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [selectedUser, setSelectedUser] = useState<any | null>(null);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    setLoading(true);
    const params = new URLSearchParams({ page: String(page), pageSize: "20" });
    if (search) params.set("search", search);

    fetch(`${API_BASE}/api/admin/users?${params}`, { credentials: "include" })
      .then(r => r.json())
      .then(data => {
        setUsers(data.data || []);
        setTotal(data.pagination?.total || 0);
      })
      .finally(() => setLoading(false));
  }, [search, page]);

  const handleUpdateUser = async (id: string, payload: { is_active?: boolean; is_admin?: boolean }) => {
    await fetch(`${API_BASE}/api/admin/users/${id}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify(payload),
    });
    setSelectedUser(null);
    setPage(1);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold text-[#151515]">Users</h1>
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

      <div className="bg-white rounded-2xl shadow overflow-hidden">
        <table className="w-full">
          <thead className="bg-[#faf8f5]">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Email</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Name</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Active</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Admin</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Created</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr><td colSpan={6} className="p-8 text-center">Loading...</td></tr>
            ) : users.length === 0 ? (
              <tr><td colSpan={6} className="p-8 text-center text-[#625b54]">No users found</td></tr>
            ) : users.map(user => (
              <tr key={user.id} className="border-t border-black/4">
                <td className="px-6 py-4 font-medium text-[#151515]">{user.email}</td>
                <td className="px-6 py-4 text-[#625b54]">{user.name || "-"}</td>
                <td className="px-6 py-4">
                  <span className={`text-xs px-2 py-1 rounded ${user.is_active ? "bg-green-100" : "bg-red-100"}`}>
                    {user.is_active ? "active" : "inactive"}
                  </span>
                </td>
                <td className="px-6 py-4">
                  <span className={`text-xs px-2 py-1 rounded ${user.is_admin ? "bg-purple-100" : "bg-gray-100"}`}>
                    {user.is_admin ? "admin" : "user"}
                  </span>
                </td>
                <td className="px-6 py-4 text-[#625b54]">{new Date(user.createdAt).toLocaleDateString()}</td>
                <td className="px-6 py-4">
                  <button onClick={() => setSelectedUser(user)} className="text-sm font-bold text-[#151515] hover:underline">
                    Edit
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
      {selectedUser && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-white rounded-3xl p-8 w-full max-w-lg max-h-[90vh] overflow-auto">
            <h2 className="text-2xl font-bold text-[#151515] mb-4">User Details</h2>
            <div className="space-y-4">
              <div><label className="text-sm text-[#625b54]">Email</label><div className="font-medium">{selectedUser.email}</div></div>
              <div><label className="text-sm text-[#625b54]">Name</label><div className="font-medium">{selectedUser.name || "-"}</div></div>
              <div><label className="text-sm text-[#625b54]">Active</label>
                <select
                  id="user-active"
                  defaultValue={selectedUser.is_active ? "true" : "false"}
                  className="coz-input w-full mt-1"
                  onChange={e => selectedUser.is_active = e.target.value === "true"}
                >
                  <option value="true">Active</option>
                  <option value="false">Inactive</option>
                </select>
              </div>
              <div><label className="text-sm text-[#625b54]">Admin</label>
                <select
                  id="user-admin"
                  defaultValue={selectedUser.is_admin ? "true" : "false"}
                  className="coz-input w-full mt-1"
                  onChange={e => selectedUser.is_admin = e.target.value === "true"}
                >
                  <option value="false">User</option>
                  <option value="true">Admin</option>
                </select>
              </div>
            </div>
            <div className="flex gap-3 mt-6">
              <button
                onClick={() => handleUpdateUser(selectedUser.id, {
                  is_active: (document.getElementById("user-active") as HTMLSelectElement).value === "true",
                  is_admin: (document.getElementById("user-admin") as HTMLSelectElement).value === "true",
                })}
                className="coz-btn-dark flex-1"
              >
                Save
              </button>
              <button onClick={() => setSelectedUser(null)} className="coz-btn-light flex-1">
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}