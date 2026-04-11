import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Demo Hub — Pulseclaw",
  description:
    "观看 Pulseclaw 如何从阅读痕迹、调试现场、多窗口切换中捕获上下文，再把证据链长成候选帮助。三个真实场景入口。",
};

export default function DemoLayout({ children }: { children: React.ReactNode }) {
  return children;
}
