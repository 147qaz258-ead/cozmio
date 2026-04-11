import type { Metadata } from "next";
import Link from "next/link";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/button";
import { getProgressPageData } from "@/lib/progress-data";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  title: "项目进展",
  description:
    "每天查看 Pulseclaw 的真实推进：任务主线、git 构建节奏、代码增量与验证状态。",
  alternates: { canonical: "/progress" },
};

function formatNum(value: number) {
  return new Intl.NumberFormat("zh-CN").format(value);
}

function formatDate(date: string) {
  const d = new Date(`${date}T00:00:00`);
  return new Intl.DateTimeFormat("zh-CN", { month: "short", day: "numeric" }).format(d);
}

function barHeight(net: number, maxNet: number) {
  if (maxNet === 0) return 4;
  return Math.max(4, Math.round((Math.abs(net) / maxNet) * 120));
}

export default function ProgressPage() {
  const d = getProgressPageData();

  const maxNet = Math.max(...d.dailyProgress.map((day) => Math.abs(day.netLines)), 1);

  const totalAdditions = d.dailyProgress.reduce((s, day) => s + day.additions, 0);
  const totalDeletions = d.dailyProgress.reduce((s, day) => s + day.deletions, 0);

  const recentDays = d.dailyProgress.slice(0, 7);

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />
      <main className="flex-1">
        {/* Hero */}
        <section className="relative overflow-hidden pb-12 pt-12 sm:pb-16 lg:pb-18 lg:pt-18">
          <div className="pointer-events-none absolute inset-0">
            <div className="page-grid absolute inset-0 opacity-[0.2]" />
            <div className="absolute left-[-6%] top-[6%] h-[22rem] w-[22rem] rounded-full bg-mist-blue/14 blur-3xl" />
            <div className="absolute right-[-8%] top-[12%] h-[26rem] w-[26rem] rounded-full bg-digital-lavender/12 blur-3xl" />
            <div className="absolute bottom-[-16%] left-[30%] h-[24rem] w-[24rem] rounded-full bg-sage-green/10 blur-3xl" />
          </div>

          <Container className="relative">
            <div className="grid gap-10 lg:grid-cols-[0.94fr_1.06fr] lg:items-end">
              <div className="max-w-[42rem]">
                <span className="section-kicker">Pulseclaw</span>
                <h1 className="mt-6 text-[clamp(2.8rem,6vw,5.2rem)] font-semibold leading-[0.96] text-primary-text">
                  迭代速度，
                  <span className="mt-3 block text-mist-blue">
                    用代码说话。
                  </span>
                </h1>
                <p className="mt-6 max-w-[38rem] text-[1.05rem] leading-8 text-secondary-text sm:text-[1.12rem]">
                  基于真实 git 构建历史生成。每个数字都来自代码增删，不来自感觉。
                </p>
                <div className="mt-8 flex gap-3">
                  <a href={siteConfig.links.github} target="_blank" rel="noreferrer" className="inline-flex">
                    <Button
                      size="lg"
                      className="h-[3.15rem] rounded-2xl bg-primary-text px-6 text-base font-semibold text-white shadow-[0_20px_44px_rgba(45,42,38,0.16)] hover:bg-primary-text/94"
                    >
                      查看仓库
                    </Button>
                  </a>
                  <Link href="/demo" className="inline-flex">
                    <Button
                      size="lg"
                      variant="outline"
                      className="h-[3.15rem] rounded-2xl border-black/8 bg-white/80 px-6 text-base font-medium text-primary-text hover:bg-white"
                    >
                      打开演示
                    </Button>
                  </Link>
                </div>
              </div>

              {/* Top-level stats */}
              <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-1 xl:grid-cols-2">
                {[
                  {
                    label: "近 7 天新增代码",
                    value: `+${formatNum(d.recentAdditions)}`,
                    unit: "行",
                    accent: "text-mist-blue",
                  },
                  {
                    label: "近 7 天删除代码",
                    value: `-${formatNum(d.recentDeletions)}`,
                    unit: "行",
                    accent: "text-secondary-text",
                  },
                  {
                    label: "近 7 天提交",
                    value: formatNum(d.recentCommitCount),
                    unit: "次",
                    accent: "text-primary-text",
                  },
                  {
                    label: "21 天净增",
                    value: `${d.recentAdditions - d.recentDeletions > 0 ? "+" : ""}${formatNum(d.recentAdditions - d.recentDeletions)}`,
                    unit: "行",
                    accent: d.recentAdditions - d.recentDeletions >= 0 ? "text-sage-green" : "text-amber-600",
                  },
                ].map((item) => (
                  <div
                    key={item.label}
                    className="surface-panel rounded-[1.4rem] border border-black/6 bg-white/80 p-4"
                  >
                    <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                      {item.label}
                    </div>
                    <div className={`mt-1 text-[2rem] font-semibold ${item.accent}`}>
                      {item.value}
                      <span className="ml-1 text-[1rem] font-normal text-secondary-text">{item.unit}</span>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </Container>
        </section>

        {/* 21-day bar chart */}
        <section className="pb-10 sm:pb-14">
          <Container>
            <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
              <div className="mb-5 border-b border-black/6 pb-4">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                  21 天提交节奏
                </div>
                <h2 className="mt-2 text-[1.6rem] font-semibold text-primary-text">
                  每天的代码增删，一目了然
                </h2>
              </div>

              <div className="flex items-end gap-1.5 overflow-x-auto pb-2">
                {d.dailyProgress.map((day) => {
                  const pos = day.netLines > 0 ? barHeight(day.netLines, maxNet) : 0;
                  const neg = day.netLines < 0 ? barHeight(day.netLines, maxNet) : 0;
                  const isToday =
                    day.date === d.dailyProgress[0]?.date;

                  return (
                    <div
                      key={day.date}
                      className="flex flex-1 flex-col items-center gap-1"
                      title={`${day.date}  +${day.additions} -${day.deletions}  ${day.commitCount} commits`}
                    >
                      <div className="flex w-full flex-col items-center justify-end" style={{ height: 120 }}>
                        {pos > 0 && (
                          <div
                            className="w-full rounded-t-sm bg-mist-blue/70"
                            style={{ height: pos }}
                          />
                        )}
                      </div>
                      <div className="flex w-full flex-col items-center justify-start" style={{ height: 60 }}>
                        {neg > 0 && (
                          <div
                            className="w-full rounded-b-sm bg-amber-500/60"
                            style={{ height: neg }}
                          />
                        )}
                      </div>
                      <div
                        className={`w-full text-center text-[9px] font-medium ${isToday ? "text-mist-blue" : "text-secondary-text/50"}`}
                      >
                        {formatDate(day.date)}
                      </div>
                    </div>
                  );
                })}
              </div>

              <div className="mt-4 flex items-center gap-5 text-[11px] text-secondary-text">
                <span className="flex items-center gap-1.5">
                  <span className="inline-block h-2 w-4 rounded-sm bg-mist-blue/70" />
                  新增
                </span>
                <span className="flex items-center gap-1.5">
                  <span className="inline-block h-2 w-4 rounded-sm bg-amber-500/60" />
                  删除
                </span>
              </div>
            </div>
          </Container>
        </section>

        {/* Recent commits */}
        <section className="pb-16 sm:pb-20">
          <Container>
            <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
              <div className="mb-5 border-b border-black/6 pb-4">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                  最新提交
                </div>
                <h2 className="mt-2 text-[1.6rem] font-semibold text-primary-text">
                  近 7 天推进了什么
                </h2>
              </div>

              <div className="space-y-3">
                {recentDays.flatMap((day) =>
                  day.commits.map((commit, ci) => (
                    <div
                      key={`${day.date}-${ci}`}
                      className="flex flex-wrap items-start gap-3 rounded-[1.2rem] border border-black/6 bg-white/80 px-4 py-3"
                    >
                      <div className="flex min-w-0 flex-1 gap-3">
                        <span className="shrink-0 rounded-full border border-black/8 bg-white/90 px-2.5 py-0.5 text-[10px] font-medium text-secondary-text">
                          {formatDate(day.date)}
                        </span>
                        <span className="min-w-0 flex-1 text-sm font-medium text-primary-text">
                          {commit.subject}
                        </span>
                      </div>
                      <div className="flex shrink-0 items-center gap-3 text-[11px]">
                        <span className="font-mono text-mist-blue">+{formatNum(commit.additions)}</span>
                        <span className="font-mono text-secondary-text/60">-{formatNum(commit.deletions)}</span>
                        <span className="rounded-full border border-black/8 bg-white/90 px-2 py-0.5 text-[10px] text-secondary-text">
                          {commit.filesChanged}f
                        </span>
                      </div>
                    </div>
                  )),
                )}

                {recentDays.length === 0 && (
                  <div className="py-8 text-center text-sm text-secondary-text">
                    暂无提交记录
                  </div>
                )}
              </div>
            </div>
          </Container>
        </section>
      </main>
      <Footer />
    </div>
  );
}
