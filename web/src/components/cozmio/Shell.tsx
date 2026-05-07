"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useState } from "react";
import { ArrowRight, GitBranch, Mail } from "lucide-react";
import { siteConfig } from "@/lib/site-config";

const nav = [
  { href: "/agents", label: "智能体" },
  { href: "/cases", label: "案例" },
  { href: "/desktop", label: "桌面节点" },
  { href: "/request", label: "提交任务" },
];

export function CozLogo() {
  return (
    <Link href="/" className="flex items-center gap-3">
      <span className="grid h-9 w-9 place-items-center rounded-full border border-[#e1d5b9] bg-white shadow-[inset_0_1px_0_rgba(255,255,255,.9)]">
        <span className="h-5 w-5 rounded-[7px] border-2 border-[#151515] bg-gradient-to-br from-[#fff9d8] to-[#9df2b2]" />
      </span>
      <span className="text-2xl font-bold tracking-tight text-[#111]">Cozmio</span>
    </Link>
  );
}

export function CozHeader() {
  const pathname = usePathname();
  return (
    <header className="coz-shell sticky top-3 z-50">
      <div className="coz-glass flex h-[58px] items-center justify-between rounded-[16px] px-5">
        <CozLogo />
        <nav className="hidden items-center gap-12 text-[13px] font-semibold text-[#1b1b1b] lg:flex">
          {nav.map((item) => {
            const active = pathname === item.href || (item.href === "/request" && pathname?.startsWith("/request"));
            return (
              <Link key={item.href} href={item.href} className="relative py-5">
                {item.label}
                {active && <span className="absolute inset-x-1 -bottom-1 h-[3px] rounded-full bg-[#111]" />}
              </Link>
            );
          })}
        </nav>
        <div className="flex items-center gap-3">
          <button className="hidden text-sm font-semibold text-[#171717] sm:block">中 <span className="text-[#999]">/ EN</span></button>
          <a href={siteConfig.links.github} className="coz-btn-light hidden h-10 items-center gap-2 rounded-full px-5 text-sm font-semibold md:flex">
            <GitBranch className="h-4 w-4" />
            GitHub
          </a>
          <Link href="/request" className="coz-btn-dark flex h-10 items-center gap-2 rounded-full px-5 text-sm font-semibold">
            提交任务
            <ArrowRight className="h-4 w-4" />
          </Link>
        </div>
      </div>
    </header>
  );
}

const footerLinks = [
  { title: "产品", links: [
    { label: "智能体", href: "/agents" },
    { label: "桌面节点", href: "/desktop" },
    { label: "提交任务", href: "/request" },
  ]},
  { title: "资源", links: [
    { label: "案例", href: "/cases" },
    { label: "文档", href: "/blog" },
    { label: "帮助中心", href: "/about" },
  ]},
  { title: "公司", links: [
    { label: "关于我们", href: "/about" },
    { label: "隐私政策", href: "/privacy" },
    { label: "服务条款", href: "/terms" },
  ]},
];

export function CozFooter() {
  const [email, setEmail] = useState("");
  const [subscribed, setSubscribed] = useState(false);
  const [loading, setLoading] = useState(false);

  const handleSubscribe = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!email) return;
    setLoading(true);
    // Mock subscribe - in production this would call an API
    await new Promise(resolve => setTimeout(resolve, 800));
    setSubscribed(true);
    setLoading(false);
  };

  return (
    <footer className="coz-shell pb-14 pt-14">
      <div className="grid gap-10 border-t border-[#1a1612]/10 pt-10 md:grid-cols-[1.3fr_2fr_1.4fr]">
        <div>
          <CozLogo />
          <p className="mt-4 text-sm text-[#776f66]">Agent Build Network</p>
          <p className="mt-10 text-xs text-[#aaa198]">© 2026 Cozmio. All rights reserved.</p>
        </div>
        <div className="grid grid-cols-3 gap-8">
          {footerLinks.map((group) => (
            <div key={group.title}>
              <h3 className="text-sm font-bold text-[#191919]">{group.title}</h3>
              <div className="mt-4 space-y-3">
                {group.links.map((link) => (
                  <Link key={link.href} href={link.href} className="block text-sm text-[#756e66] transition-colors hover:text-[#151515]">
                    {link.label}
                  </Link>
                ))}
              </div>
            </div>
          ))}
        </div>
        <div>
          <h3 className="text-sm font-bold text-[#191919]">订阅更新</h3>
          <p className="mt-3 text-sm text-[#756e66]">获取产品更新与网络动态</p>
          {subscribed ? (
            <p className="mt-5 text-sm text-green-600">订阅成功！</p>
          ) : (
            <form onSubmit={handleSubscribe} className="mt-5">
              <div className="flex h-12 items-center rounded-2xl border border-[#1a1612]/10 bg-white/60 px-4">
                <input
                  type="email"
                  value={email}
                  onChange={e => setEmail(e.target.value)}
                  placeholder="输入你的邮箱"
                  className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-[#aaa]"
                  required
                />
                <button type="submit" disabled={loading} className="text-[#888] hover:text-[#151515] disabled:opacity-50">
                  <Mail className="h-4 w-4" />
                </button>
              </div>
              {loading && <p className="mt-2 text-xs text-[#756e66]">订阅中...</p>}
            </form>
          )}
          <div className="mt-6 flex gap-3">
            <a href="https://github.com/147qaz258-ead/cozmio" target="_blank" rel="noreferrer" className="grid h-10 w-10 place-items-center rounded-full border border-[#1a1612]/10 bg-white/65 text-xs font-bold text-[#151515] hover:bg-white">GH</a>
            <a href="https://x.com/wjnhng419090" target="_blank" rel="noreferrer" className="grid h-10 w-10 place-items-center rounded-full border border-[#1a1612]/10 bg-white/65 text-xs font-bold text-[#151515] hover:bg-white">X</a>
            <a href="https://discord.gg/cozmio" target="_blank" rel="noreferrer" className="grid h-10 w-10 place-items-center rounded-full border border-[#1a1612]/10 bg-white/65 text-xs font-bold text-[#151515] hover:bg-white">DC</a>
          </div>
        </div>
      </div>
    </footer>
  );
}

export function CozPageShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="coz-page min-h-screen text-[#151515]">
      <div className="coz-light-field" />
      <CozHeader />
      <main>{children}</main>
      <CozFooter />
    </div>
  );
}
