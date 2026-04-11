import type { Metadata } from "next";
import Link from "next/link";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/button";
import { getProgressPageData, type DailyProgressDay } from "@/lib/progress-data";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  title: "项目进展",
  description:
    "每天查看 Pulseclaw 的真实推进：任务主线、git 构建节奏、代码增量与验证状态。",
  alternates: {
    canonical: "/progress",
  },
};

function formatNumber(value: number) {
  return new Intl.NumberFormat("zh-CN").format(value);
}

function formatPercent(value: number) {
  return `${Math.round(value * 100)}%`;
}

function formatSigned(value: number) {
  const sign = value > 0 ? "+" : "";
  return `${sign}${formatNumber(value)}`;
}

function formatDateLabel(date: string) {
  const asDate = new Date(`${date}T00:00:00`);
  return new Intl.DateTimeFormat("zh-CN", {
    month: "short",
    day: "numeric",
    weekday: "short",
  }).format(asDate);
}

function formatGeneratedAt(value: string) {
  const asDate = new Date(value);
  return new Intl.DateTimeFormat("zh-CN", {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  }).format(asDate);
}

function statusLabel(status: string, verificationStatus?: string) {
  if (status === "passed") return "verified";
  if (status === "in_progress") return "running";
  if (status === "implemented_unverified") return "implemented";
  if (status === "blocked" || verificationStatus === "blocked") return "blocked";
  if (status === "failed" || verificationStatus === "failed") return "attention";
  return "queued";
}

function statusTone(status: string, verificationStatus?: string) {
  if (status === "passed") return "border-sage-green/22 bg-sage-green/10 text-sage-green";
  if (status === "in_progress")
    return "border-mist-blue/24 bg-mist-blue/10 text-mist-blue";
  if (status === "implemented_unverified")
    return "border-digital-lavender/26 bg-digital-lavender/10 text-digital-lavender";
  if (status === "blocked" || verificationStatus === "blocked")
    return "border-primary-text/12 bg-primary-text/8 text-primary-text";
  if (status === "failed" || verificationStatus === "failed")
    return "border-amber-500/28 bg-amber-500/10 text-amber-700";
  return "border-black/8 bg-white text-secondary-text";
}

function renderDailyTree(day: DailyProgressDay, isLast: boolean) {
  const branch = isLast ? "└─" : "├─";
  const childPrefix = isLast ? "   " : "│  ";

  return (
    <div key={day.date} className="font-mono text-[12px] leading-6 text-primary-text">
      <div className="flex flex-wrap items-center gap-3">
        <span className="text-secondary-text">{branch}</span>
        <span className="font-semibold text-primary-text">{day.date}</span>
        <span className="rounded-full border border-black/8 bg-white/80 px-2 py-0.5 text-[11px] text-secondary-text">
          {formatDateLabel(day.date)}
        </span>
        <span className="text-mist-blue">+{formatNumber(day.additions)}</span>
        <span className="text-primary-text/60">-{formatNumber(day.deletions)}</span>
        <span className="text-secondary-text">{formatNumber(day.commitCount)} commits</span>
      </div>

      {day.commits.map((commit) => (
        <div key={commit.sha} className="flex flex-wrap items-center gap-3">
          <span className="text-secondary-text">{childPrefix}├─</span>
          <span className="text-primary-text/84">{commit.subject}</span>
          <span className="text-secondary-text/72">{commit.sha.slice(0, 7)}</span>
          <span className="text-mist-blue">+{formatNumber(commit.additions)}</span>
          <span className="text-primary-text/60">-{formatNumber(commit.deletions)}</span>
        </div>
      ))}

      <div className="flex flex-wrap items-center gap-3">
        <span className="text-secondary-text">{childPrefix}└─</span>
        <span className="text-secondary-text">areas</span>
        <span className="text-primary-text/80">{day.areas.join(" · ") || "repo-wide"}</span>
      </div>
    </div>
  );
}

export default function ProgressPage() {
  const data = getProgressPageData();

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />
      <main className="flex-1">
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
                  把每天的推进，
                  <span className="mt-3 block text-mist-blue">变成一页能看见的构建轨迹。</span>
                </h1>
                <p className="mt-6 max-w-[38rem] text-[1.05rem] leading-8 text-secondary-text sm:text-[1.12rem]">
                  这不是一张只有提交点的活跃图。{data.codename} 同时读取任务主线和
                  git 构建历史，让你看到今天推进了什么、改了哪些区域、加了多少代码，
                  以及主线到底走到了哪。
                </p>

                <div className="mt-8 flex flex-col gap-3 sm:flex-row">
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

              <div className="surface-panel-strong grid gap-4 rounded-[2rem] p-5 sm:grid-cols-2 sm:p-6">
                <div className="rounded-[1.4rem] border border-black/6 bg-white/80 p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    主线完成度
                  </div>
                  <div className="mt-3 text-[2.4rem] font-semibold text-primary-text">
                    {formatPercent(data.completionRate)}
                  </div>
                  <div className="mt-2 text-sm text-secondary-text">
                    {formatNumber(data.passedTasks)} / {formatNumber(data.totalTasks)} tasks verified
                  </div>
                </div>

                <div className="rounded-[1.4rem] border border-black/6 bg-white/80 p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    最近 7 天提交
                  </div>
                  <div className="mt-3 text-[2.4rem] font-semibold text-primary-text">
                    {formatNumber(data.recentCommitCount)}
                  </div>
                  <div className="mt-2 text-sm text-secondary-text">
                    +{formatNumber(data.recentAdditions)} / -{formatNumber(data.recentDeletions)} lines
                  </div>
                </div>

                <div className="rounded-[1.4rem] border border-black/6 bg-white/80 p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    当前活跃面
                  </div>
                  <div className="mt-3 text-[2.4rem] font-semibold text-primary-text">
                    {formatNumber(data.activeTasks)}
                  </div>
                  <div className="mt-2 text-sm text-secondary-text">
                    包含 implemented 与 in-progress
                  </div>
                </div>

                <div className="rounded-[1.4rem] border border-black/6 bg-white/80 p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    最后生成
                  </div>
                  <div className="mt-3 text-[1.6rem] font-semibold text-primary-text">
                    {formatGeneratedAt(data.generatedAt)}
                  </div>
                  <div className="mt-2 text-sm text-secondary-text">
                    tasklist {data.tasklistVersion}
                  </div>
                </div>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-12 sm:pb-16">
          <Container>
            <div className="grid gap-6 xl:grid-cols-[1.08fr_0.92fr]">
              <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
                <div className="flex flex-wrap items-start justify-between gap-4 border-b border-black/6 pb-4">
                  <div>
                    <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                      Daily Build Tree
                    </div>
                    <h2 className="mt-2 text-[1.7rem] font-semibold text-primary-text">
                      每天加了什么，改了哪里，多了多少代码
                    </h2>
                    <p className="mt-3 max-w-[38rem] text-sm leading-7 text-secondary-text">
                      这里直接基于 git 构建历史生成。每个日期节点都会显示代码增减、
                      提交主题和当天最活跃的改动区域。
                    </p>
                  </div>
                  <div className="rounded-[1.25rem] border border-black/6 bg-white/80 px-4 py-3 text-right">
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                      last 21 days
                    </div>
                    <div className="mt-1 text-[1.35rem] font-semibold text-primary-text">
                      {formatNumber(data.dailyProgress.length)} day nodes
                    </div>
                  </div>
                </div>

                <div className="mt-5 rounded-[1.6rem] border border-black/6 bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(246,242,236,0.95))] p-4 sm:p-5">
                  <div className="font-mono text-[12px] leading-6 text-primary-text">
                    <div className="font-semibold text-primary-text">pulseclaw/</div>
                    <div className="mt-3 space-y-2">
                      {data.dailyProgress.map((day, index) =>
                        renderDailyTree(day, index === data.dailyProgress.length - 1),
                      )}
                    </div>
                  </div>
                </div>
              </div>

              <div className="space-y-6">
                <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    Current Frontier
                  </div>
                  <h2 className="mt-2 text-[1.55rem] font-semibold text-primary-text">
                    当前主线正在碰哪几块
                  </h2>
                  <div className="mt-5 space-y-3">
                    {data.currentFrontier.map((task) => (
                      <div
                        key={task.id}
                        className="rounded-[1.35rem] border border-black/6 bg-white/80 p-4"
                      >
                        <div className="flex flex-wrap items-center gap-2">
                          <span className="font-mono text-[12px] font-semibold text-primary-text">
                            {task.id}
                          </span>
                          <span
                            className={`rounded-full border px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] ${statusTone(
                              task.status,
                              task.verification_status,
                            )}`}
                          >
                            {statusLabel(task.status, task.verification_status)}
                          </span>
                        </div>
                        <div className="mt-2 text-sm font-semibold text-primary-text">
                          {task.title}
                        </div>
                        {task.notes ? (
                          <div className="mt-2 text-sm leading-7 text-secondary-text">
                            {task.notes.split(/\r?\n/)[0]}
                          </div>
                        ) : null}
                      </div>
                    ))}
                  </div>
                </div>

                <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    Repo Pulse
                  </div>
                  <h2 className="mt-2 text-[1.55rem] font-semibold text-primary-text">
                    这段时间的推进节奏
                  </h2>
                  <div className="mt-5 grid gap-3 sm:grid-cols-2">
                    {[
                      {
                        label: "blocked tasks",
                        value: formatNumber(data.blockedTasks),
                        tone: "text-primary-text",
                      },
                      {
                        label: "net lines / 7d",
                        value: formatSigned(data.recentAdditions - data.recentDeletions),
                        tone: "text-mist-blue",
                      },
                      {
                        label: "latest ship day",
                        value: data.dailyProgress[0]?.date ?? "n/a",
                        tone: "text-primary-text",
                      },
                      {
                        label: "brand",
                        value: data.codename,
                        tone: "text-digital-lavender",
                      },
                    ].map((item) => (
                      <div
                        key={item.label}
                        className="rounded-[1.35rem] border border-black/6 bg-white/80 p-4"
                      >
                        <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                          {item.label}
                        </div>
                        <div className={`mt-2 text-[1.35rem] font-semibold ${item.tone}`}>
                          {item.value}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-18 sm:pb-24">
          <Container>
            <div className="surface-panel rounded-[1.95rem] p-5 sm:p-6">
              <div className="flex flex-wrap items-start justify-between gap-4 border-b border-black/6 pb-4">
                <div>
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                    Task Tree
                  </div>
                  <h2 className="mt-2 text-[1.7rem] font-semibold text-primary-text">
                    主线任务树，不再靠感觉猜项目走到了哪
                  </h2>
                  <p className="mt-3 max-w-[44rem] text-sm leading-7 text-secondary-text">
                    这棵树直接读取任务表当前状态。lane 是主干，任务是分支；
                    每个节点都标出它是 verified、implemented、blocked 还是 queued。
                  </p>
                </div>
                <div className="rounded-[1.25rem] border border-black/6 bg-white/80 px-4 py-3 text-right">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                    verified now
                  </div>
                  <div className="mt-1 text-[1.35rem] font-semibold text-primary-text">
                    {formatNumber(data.passedTasks)} tasks
                  </div>
                </div>
              </div>

              <div className="mt-5 rounded-[1.6rem] border border-black/6 bg-[linear-gradient(180deg,rgba(255,255,255,0.96),rgba(246,242,236,0.95))] p-4 sm:p-5">
                <div className="space-y-5 font-mono text-[12px] leading-6 text-primary-text">
                  <div className="font-semibold text-primary-text">pulseclaw/</div>
                  {data.laneSnapshots.map((lane, laneIndex) => {
                    const lanePrefix = laneIndex === data.laneSnapshots.length - 1 ? "└─" : "├─";
                    const laneChildPrefix =
                      laneIndex === data.laneSnapshots.length - 1 ? "   " : "│  ";

                    return (
                      <div key={lane.id}>
                        <div className="flex flex-wrap items-center gap-3">
                          <span className="text-secondary-text">{lanePrefix}</span>
                          <span className="font-semibold text-primary-text">
                            {lane.id} · {lane.name}
                          </span>
                          <span className="rounded-full border border-black/8 bg-white/80 px-2 py-0.5 text-[11px] text-secondary-text">
                            {lane.passed}/{lane.total} verified
                          </span>
                        </div>
                        <div className="ml-6 mt-1 text-sm leading-7 text-secondary-text">
                          {lane.description}
                        </div>

                        <div className="mt-2 space-y-1">
                          {lane.tasks.map((task, taskIndex) => {
                            const taskPrefix =
                              taskIndex === lane.tasks.length - 1 ? "└─" : "├─";
                            return (
                              <div key={task.id} className="flex flex-wrap items-center gap-3">
                                <span className="text-secondary-text">
                                  {laneChildPrefix}
                                  {taskPrefix}
                                </span>
                                <span className="font-semibold text-primary-text">
                                  {task.id}
                                </span>
                                <span
                                  className={`rounded-full border px-2 py-0.5 text-[10px] font-semibold uppercase tracking-[0.16em] ${statusTone(
                                    task.status,
                                    task.verification_status,
                                  )}`}
                                >
                                  {statusLabel(task.status, task.verification_status)}
                                </span>
                                <span className="text-primary-text/84">{task.title}</span>
                              </div>
                            );
                          })}
                        </div>
                      </div>
                    );
                  })}
                </div>
              </div>
            </div>
          </Container>
        </section>
      </main>
      <Footer />
    </div>
  );
}
