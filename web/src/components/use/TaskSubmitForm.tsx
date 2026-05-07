"use client";

import { useState } from "react";
import { createTask } from "@/lib/api";

interface Props {
  onSuccess: () => void;
  onCancel: () => void;
}

export function TaskSubmitForm({ onSuccess, onCancel }: Props) {
  const [title, setTitle] = useState("");
  const [prompt, setPrompt] = useState("");
  const [sourceUrl, setSourceUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);
    try {
      await createTask({ title, prompt, sourceUrl: sourceUrl || undefined });
      onSuccess();
    } catch (err: any) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="bg-white rounded-lg shadow p-6">
      <h2 className="text-lg font-semibold text-warm-900 mb-4">Submit New Task</h2>
      <form onSubmit={handleSubmit} className="space-y-4">
        <div>
          <label className="block text-sm font-medium text-warm-700 mb-1">Title</label>
          <input
            type="text"
            value={title}
            onChange={e => setTitle(e.target.value)}
            className="w-full px-3 py-2 border border-warm-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            required
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-warm-700 mb-1">Prompt</label>
          <textarea
            value={prompt}
            onChange={e => setPrompt(e.target.value)}
            rows={4}
            className="w-full px-3 py-2 border border-warm-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
            required
          />
        </div>
        <div>
          <label className="block text-sm font-medium text-warm-700 mb-1">Source URL (optional)</label>
          <input
            type="url"
            value={sourceUrl}
            onChange={e => setSourceUrl(e.target.value)}
            className="w-full px-3 py-2 border border-warm-200 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
        </div>
        {error && <p className="text-sm text-red-600">{error}</p>}
        <div className="flex gap-3">
          <button
            type="submit"
            disabled={loading}
            className="px-4 py-2 bg-primary-600 text-white rounded-lg hover:bg-primary-700 disabled:opacity-50"
          >
            {loading ? "Submitting..." : "Submit Task"}
          </button>
          <button
            type="button"
            onClick={onCancel}
            className="px-4 py-2 border border-warm-300 text-warm-700 rounded-lg hover:bg-warm-50"
          >
            Cancel
          </button>
        </div>
      </form>
    </div>
  );
}