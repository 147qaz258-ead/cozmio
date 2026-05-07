"use client";

import { useState, useEffect } from "react";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

export default function DownloadsPage() {
  const [downloads, setDownloads] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [selectedDownload, setSelectedDownload] = useState<any | null>(null);
  const [isCreating, setIsCreating] = useState(false);
  const [page, setPage] = useState(1);
  const [total, setTotal] = useState(0);

  useEffect(() => {
    setLoading(true);
    const params = new URLSearchParams({ page: String(page), pageSize: "20" });
    if (search) params.set("search", search);

    fetch(`${API_BASE}/api/admin/downloads?${params}`, { credentials: "include" })
      .then(r => r.json())
      .then(data => {
        setDownloads(data.data || []);
        setTotal(data.pagination?.total || 0);
      })
      .finally(() => setLoading(false));
  }, [search, page]);

  const handleCreate = async (payload: { title: string; url: string; platform: string; version: string }) => {
    await fetch(`${API_BASE}/api/admin/downloads`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify(payload),
    });
    setIsCreating(false);
    setPage(1);
  };

  const handleUpdate = async (id: string, payload: { title?: string; url?: string; platform?: string; version?: string }) => {
    await fetch(`${API_BASE}/api/admin/downloads/${id}`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      credentials: "include",
      body: JSON.stringify(payload),
    });
    setSelectedDownload(null);
    setPage(1);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-6">
        <h1 className="text-3xl font-bold text-[#151515]">Downloads</h1>
        <div className="flex items-center gap-4">
          <input
            type="text"
            placeholder="Search..."
            value={search}
            onChange={e => setSearch(e.target.value)}
            className="coz-input"
          />
          <button onClick={() => setIsCreating(true)} className="coz-btn-dark">
            + Add Download
          </button>
        </div>
      </div>

      <div className="bg-white rounded-2xl shadow overflow-hidden">
        <table className="w-full">
          <thead className="bg-[#faf8f5]">
            <tr>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Title</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Platform</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Version</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">URL</th>
              <th className="px-6 py-3 text-left text-xs font-bold text-[#625b54]">Actions</th>
            </tr>
          </thead>
          <tbody>
            {loading ? (
              <tr><td colSpan={5} className="p-8 text-center">Loading...</td></tr>
            ) : downloads.length === 0 ? (
              <tr><td colSpan={5} className="p-8 text-center text-[#625b54]">No downloads found</td></tr>
            ) : downloads.map(dl => (
              <tr key={dl.id} className="border-t border-black/4">
                <td className="px-6 py-4 font-medium text-[#151515]">{dl.title}</td>
                <td className="px-6 py-4 text-[#625b54]">{dl.platform}</td>
                <td className="px-6 py-4 text-[#625b54]">{dl.version}</td>
                <td className="px-6 py-4 text-[#625b54] truncate max-w-xs">{dl.url}</td>
                <td className="px-6 py-4">
                  <button onClick={() => setSelectedDownload(dl)} className="text-sm font-bold text-[#151515] hover:underline">
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

      {/* Create Modal */}
      {isCreating && (
        <DownloadModal
          onClose={() => setIsCreating(false)}
          onSave={handleCreate}
        />
      )}

      {/* Edit Modal */}
      {selectedDownload && (
        <DownloadModal
          download={selectedDownload}
          onClose={() => setSelectedDownload(null)}
          onSave={(payload) => handleUpdate(selectedDownload.id, payload)}
        />
      )}
    </div>
  );
}

function DownloadModal({ download, onClose, onSave }: { download?: any; onClose: () => void; onSave: (p: any) => void }) {
  const [title, setTitle] = useState(download?.title || "");
  const [url, setUrl] = useState(download?.url || "");
  const [platform, setPlatform] = useState(download?.platform || "");
  const [version, setVersion] = useState(download?.version || "");

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white rounded-3xl p-8 w-full max-w-lg max-h-[90vh] overflow-auto">
        <h2 className="text-2xl font-bold text-[#151515] mb-4">{download ? "Edit Download" : "Add Download"}</h2>
        <div className="space-y-4">
          <div>
            <label className="text-sm text-[#625b54]">Title</label>
            <input type="text" value={title} onChange={e => setTitle(e.target.value)} className="coz-input w-full mt-1" />
          </div>
          <div>
            <label className="text-sm text-[#625b54]">URL</label>
            <input type="text" value={url} onChange={e => setUrl(e.target.value)} className="coz-input w-full mt-1" />
          </div>
          <div>
            <label className="text-sm text-[#625b54]">Platform</label>
            <input type="text" value={platform} onChange={e => setPlatform(e.target.value)} className="coz-input w-full mt-1" placeholder="Windows, macOS, Linux..." />
          </div>
          <div>
            <label className="text-sm text-[#625b54]">Version</label>
            <input type="text" value={version} onChange={e => setVersion(e.target.value)} className="coz-input w-full mt-1" placeholder="1.0.0" />
          </div>
        </div>
        <div className="flex gap-3 mt-6">
          <button onClick={() => onSave({ title, url, platform, version })} className="coz-btn-dark flex-1">
            Save
          </button>
          <button onClick={onClose} className="coz-btn-light flex-1">
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}