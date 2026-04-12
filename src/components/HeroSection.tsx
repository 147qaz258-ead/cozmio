"use client";

import Link from "next/link";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/button";
import { HomepageTeaser } from "@/components/HomepageTeaser";

const TRUST_SIGNALS = [
  "Desktop-first product",
  "Local-first runtime",
  "Append-only raw evidence",
  "Replayable context",
  "Candidate help, not truth",
];

export function HeroSection() {
  return (
    <section className="relative overflow-hidden pb-18 pt-12 sm:pb-24 lg:pb-28 lg:pt-20">
      <div className="pointer-events-none absolute inset-0">
        <div className="page-grid absolute inset-0 opacity-[0.24]" />
        <div className="absolute left-[-8%] top-[10%] h-[24rem] w-[24rem] rounded-full bg-digital-lavender/12 blur-3xl" />
        <div className="absolute right-[-6%] top-[4%] h-[28rem] w-[28rem] rounded-full bg-mist-blue/14 blur-3xl" />
        <div className="absolute bottom-[-18%] left-[28%] h-[24rem] w-[24rem] rounded-full bg-sage-green/10 blur-3xl" />
      </div>

      <Container className="relative">
        <div className="grid gap-14 lg:grid-cols-[minmax(0,0.96fr)_minmax(560px,1.04fr)] lg:items-center">
          <div className="max-w-[640px]">
            <span className="section-kicker">旗舰叙事</span>

            <h1 className="mt-7 text-[clamp(3rem,7.3vw,5.75rem)] font-semibold leading-[0.96] text-primary-text">
              先保留你刚经历过的上下文，
              <span className="mt-4 block text-mist-blue">
                再让 AI 开口。
              </span>
            </h1>

            <p className="mt-7 max-w-[40rem] text-[1.12rem] leading-8 text-secondary-text sm:text-[1.18rem]">
              当你反复看一篇想学习的文章、停留在标题与封面、切回自己的灵感库又回到创作台，
              Pulseclaw 不要求你先把这一切翻译成提示词。它先把真实痕迹保留下来，再在证据支撑下提出候选帮助。
            </p>

            <div className="mt-8 grid gap-3 sm:grid-cols-2">
              {TRUST_SIGNALS.map((signal) => (
                <div
                  key={signal}
                  className="rounded-2xl border border-black/6 bg-white/68 px-4 py-3 text-sm font-medium text-primary-text shadow-[0_14px_30px_rgba(45,42,38,0.05)]"
                >
                  <span className="mr-3 inline-flex h-2.5 w-2.5 rounded-full bg-mist-blue ambient-pulse" />
                  {signal}
                </div>
              ))}
            </div>

            <div className="mt-8 flex flex-col gap-3 sm:flex-row">
              <Link href="/demo" className="inline-flex">
                <Button
                  size="lg"
                  className="h-[3.25rem] rounded-2xl bg-primary-text px-6 text-base font-semibold text-white shadow-[0_22px_50px_rgba(45,42,38,0.18)] transition-transform duration-300 hover:-translate-y-0.5 hover:bg-primary-text/94"
                >
                  运行旗舰演示
                </Button>
              </Link>

              <a href="#principles" className="inline-flex">
                <Button
                  size="lg"
                  variant="outline"
                  className="h-[3.25rem] rounded-2xl border-black/8 bg-white/75 px-6 text-base font-medium text-primary-text shadow-[0_16px_36px_rgba(45,42,38,0.06)] hover:bg-white"
                >
                  查看证据与边界
                </Button>
              </a>
            </div>

            <div className="surface-panel mt-8 rounded-[1.75rem] p-5 sm:p-6">
              <div className="flex flex-wrap items-center gap-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/72">
                <span className="rounded-full border border-sage-green/18 bg-sage-green/10 px-2.5 py-1 text-sage-green">
                  context capture
                </span>
                <span>桌面产品官网 · 公开演示与产品思考入口</span>
              </div>

              <div className="mt-4 text-xl font-semibold leading-8 text-primary-text">
                工作里的上下文，会先于提示词出现。
              </div>
              <p className="mt-3 text-base leading-7 text-secondary-text">
                阅读、停留、回看、切换，这些动作本来就在表达下一步。
                Pulseclaw 先把这些片段保留下来，再把帮助建立在证据上。
              </p>
            </div>
          </div>

          <HomepageTeaser />
        </div>
      </Container>
    </section>
  );
}
