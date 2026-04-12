"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Container } from "@/components/layout/Container";

export function CTASection() {
  return (
    <section className="pb-20 pt-6 sm:pb-24 lg:pb-28">
      <Container>
        <div className="surface-panel-strong rounded-[2rem] p-6 sm:p-8 lg:p-10">
          <div className="grid gap-8 lg:grid-cols-[1fr_auto] lg:items-end">
            <div className="max-w-[44rem]">
              <span className="section-kicker">Next Step</span>
              <h2 className="mt-6 text-[clamp(2.15rem,4vw,3.25rem)] font-semibold leading-[1.05] text-primary-text">
                看一段上下文，如何自然长成帮助。
              </h2>
              <p className="mt-5 text-[1rem] leading-7 text-secondary-text sm:text-[1.06rem]">
                一个入口展示内容场景如何沉淀成写作输出，另一个入口展示调试链路如何被完整保留下来。
                两条路径合在一起，就是 Pulseclaw 的产品表达。
              </p>

              <div className="mt-6 flex flex-wrap gap-2.5">
                {[
                  "模拟工作片段驱动",
                  "evidence-first 边界",
                  "desktop-first product",
                ].map((label) => (
                  <span
                    key={label}
                    className="rounded-full border border-black/6 bg-white/76 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-secondary-text/74"
                  >
                    {label}
                  </span>
                ))}
              </div>
            </div>

            <div className="flex flex-col gap-3 sm:flex-row lg:flex-col">
              <Link href="/demo" className="inline-flex">
                <Button
                  size="lg"
                  className="h-12 rounded-2xl bg-primary-text px-6 text-base font-semibold text-white shadow-[0_20px_46px_rgba(45,42,38,0.16)] hover:bg-primary-text/94"
                >
                  运行旗舰演示
                </Button>
              </Link>

              <Link href="/demo/debug-bug" className="inline-flex">
                <Button
                  size="lg"
                  variant="outline"
                  className="h-12 rounded-2xl border-black/8 bg-white/78 px-6 text-base font-medium text-primary-text shadow-[0_16px_32px_rgba(45,42,38,0.06)] hover:bg-white"
                >
                  查看证据证明页
                </Button>
              </Link>

              <Link href="/blog" className="inline-flex">
                <Button
                  size="lg"
                  variant="outline"
                  className="h-12 rounded-2xl border-black/8 bg-white/78 px-6 text-base font-medium text-primary-text shadow-[0_16px_32px_rgba(45,42,38,0.06)] hover:bg-white"
                >
                  阅读产品思考
                </Button>
              </Link>

              <a
                href="https://github.com/147qaz258-ead/Pulseclaw/releases/latest/download/pulseclaw-setup.exe"
                className="inline-flex"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Button
                  size="lg"
                  className="h-12 rounded-2xl bg-green-600 px-6 text-base font-semibold text-white shadow-[0_16px_32px_rgba(45,42,38,0.06)] hover:bg-green-600/94"
                >
                  下载 Windows 安装包
                </Button>
              </a>
            </div>
          </div>
        </div>
      </Container>
    </section>
  );
}
