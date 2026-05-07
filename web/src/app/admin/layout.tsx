"use client";

import { useState, useEffect } from "react";
import { useRouter, usePathname } from "next/navigation";
import Link from "next/link";

const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

const NAV_ITEMS = [
  { href: "/admin/dashboard", label: "Dashboard", icon: "📊" },
  { href: "/admin/applications", label: "Applications", icon: "📋" },
  { href: "/admin/tasks", label: "Tasks", icon: "✓" },
  { href: "/admin/users", label: "Users", icon: "👤" },
  { href: "/admin/downloads", label: "Downloads", icon: "⬇" },
];

export default function AdminLayout({ children }: { children: React.ReactNode }) {
  const [admin, setAdmin] = useState<{ email: string } | null>(null);
  const [loading, setLoading] = useState(true);
  const router = useRouter();
  const pathname = usePathname();

  useEffect(() => {
    fetch(`${API_BASE}/api/admin/auth/me`, { credentials: "include" })
      .then(res => res.json())
      .then(data => {
        if (data.ok) setAdmin(data.data);
        else router.push("/admin");
      })
      .catch(() => router.push("/admin"))
      .finally(() => setLoading(false));
  }, [router]);

  const handleLogout = async () => {
    await fetch(`${API_BASE}/api/admin/auth/logout`, { method: "POST", credentials: "include" });
    router.push("/admin");
  };

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-gray-200 border-t-[#151515]" />
      </div>
    );
  }

  if (!admin) return null;

  return (
    <div className="flex min-h-screen bg-[#faf8f5]">
      {/* Sidebar */}
      <aside className="fixed left-0 top-0 h-full w-64 bg-white border-r border-black/6">
        <div className="p-6 border-b border-black/6">
          <h1 className="text-xl font-bold text-[#151515]">Cozmio Admin</h1>
        </div>
        <nav className="p-4 space-y-1">
          {NAV_ITEMS.map(item => (
            <Link
              key={item.href}
              href={item.href}
              className={`flex items-center gap-3 rounded-xl px-4 py-3 text-sm font-bold transition-colors ${
                pathname === item.href
                  ? "bg-[#151515] text-white"
                  : "text-[#625b54] hover:bg-[#f0ece6]"
              }`}
            >
              <span>{item.icon}</span>
              {item.label}
            </Link>
          ))}
        </nav>
        <div className="absolute bottom-0 left-0 w-64 border-t border-black/6 p-4">
          <div className="flex items-center justify-between">
            <span className="text-sm text-[#625b54] truncate">{admin.email}</span>
            <button onClick={handleLogout} className="text-sm text-[#625b54] hover:text-[#151515]">
              Logout
            </button>
          </div>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 ml-64 p-8 overflow-auto">
        {children}
      </main>
    </div>
  );
}