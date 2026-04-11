"use client";

import { useEffect, useRef, useState } from "react";
import Link from "next/link";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { Button } from "@/components/ui/button";
import { ScenarioCard } from "@/components/demo/ScenarioCard";
import { ContextFlowVisual } from "@/components/demo/ContextFlowVisual";

type HubStep = {
  id: string;
  phase: string;
  title: string;
  note: string;
  sceneDescription: string;
  sceneSignals: string[];
  capsuleTitle: string;
  capsuleDescription: string;
  raw: string[];
  outputDescription: string;
  candidate: string[];
  assistantLine?: string;
};

const HUB_STEPS: HubStep[] = [
  {
    id: "observe",
    phase: "接住",
    title: "阅读痕迹进入系统",
    note: "真实上下文已经进入系统，开始形成第一层工作底座。",
    sceneDescription: "标题、封面与导语被反复查看，用户随后切回自己的灵感收藏页。",
    sceneSignals: ["标题反复回看", "封面区域停留", "切回灵感收藏页"],
    capsuleTitle: "刚发生的工作片段",
    capsuleDescription: "窗口、关键帧与停留动作开始收束成一段可回放记录。",
    raw: ["window • 浏览器文章页获得焦点", "frame • 标题区域重复停留", "event • 切回灵感收藏页"],
    outputDescription: "候选层还没有打开，系统先把上下文保存下来。",
    candidate: ["候选输出尚未出现", "原始证据正在累积", "帮助会晚于证据"],
  },
  {
    id: "preserve",
    phase: "保留",
    title: "证据包开始成形",
    note: "原始信号获得顺序、边界和可回放性，开始形成稳定的证据包。",
    sceneDescription: "窗口顺序、关键帧、回看行为和时间切面被稳定收束。",
    sceneSignals: ["浏览器与灵感库切换", "关键帧进入本地链路", "时间顺序稳定写入"],
    capsuleTitle: "保留的上下文",
    capsuleDescription: "这段经历现在具备来源关系和时间窗口，后续帮助可以回指到这里。",
    raw: ["原始 · 只增不减，数据留在本地", "追溯 · 每条帮助都可回指证据", "可回放 · 可按时间窗口重看"],
    outputDescription: "候选层仍在等待，原始证据站到舞台中央。",
    candidate: ["候选层暂未打开", "原始证据已可回放", "来源关系已经建立"],
  },
  {
    id: "condense",
    phase: "整理",
    title: "上下文胶囊形成",
    note: "系统开始组织上下文，并把关注点收束成一个可引用的工作单元。",
    sceneDescription: "当前证据正在表达：用户关注的是标题力度、封面气质与结构节奏。",
    sceneSignals: ["标题力度浮现", "封面气质被标记", "结构节奏被提取"],
    capsuleTitle: "可延续的工作单元",
    capsuleDescription: "证据被整理成一个可引用、可回放、可验证的上下文单元。",
    raw: ["聚焦 · 标题力度", "聚焦 · 封面气质", "聚焦 · 结构节奏", "可回放 · 原始证据仍可核对"],
    outputDescription: "候选帮助开始露面，但仍然处在边界之内。",
    candidate: ["标题力度拆解", "封面气质关键词", "结构节奏笔记", "一版候选提纲"],
  },
  {
    id: "assist",
    phase: "帮助",
    title: "候选帮助接住下一步",
    note: "候选帮助顺着证据链展开，直接接住下一步工作。",
    sceneDescription: "系统把刚才这段经历转成一份可以直接继续创作的写作起点。",
    sceneSignals: ["证据链已经闭合", "上下文胶囊已形成", "候选输出开始展开"],
    capsuleTitle: "准备给出帮助",
    capsuleDescription: "所有输出仍然保持可回指、可核对、可重新组织。",
    raw: ["可回放 · 仍然可以回看原始片段", "有来源 · 候选帮助附带说明依据", "有边界 · 仍未宣称理解全部目标"],
    outputDescription: "现在出现的是一组可以直接继续工作的候选结果。",
    candidate: ["公众号写作 brief", "三种标题语气候选", "封面关键词建议", "写作节奏提醒"],
    assistantLine: "基于刚才这段阅读证据，生成一份适合你当前风格的写作 brief。",
  },
];

const SCENARIOS = [
  {
    icon: "✦",
    title: "内容场景",
    description: "从阅读痕迹到写作起点，一次看完上下文如何长成输出。",
    detail: "阅读、停留、切换和回看被收束成证据包，然后在边界之内长成候选帮助。",
    status: "featured" as const,
    href: "#flagship-demo",
    badge: "featured",
    ctaLabel: "进入场景",
  },
  {
    icon: "⟡",
    title: "调试回放",
    description: "把一次崩溃现场还原成可验证、可回放的调试链路。",
    detail: "编辑器帧、终端错误、查阅动作和验证结果进入同一条本地证据链。",
    status: "active" as const,
    href: "/demo/debug-bug",
    badge: "demo",
    ctaLabel: "打开回放",
  },
  {
    icon: "◌",
    title: "多窗口工作流",
    description: "下一步会延展到邮件、文档、表格和浏览器的长链上下文。",
    detail: "更长的工作流会继续沿用同一套以证据为本、以候选为边界的产品语言。",
    status: "coming-soon" as const,
    badge: "next",
  },
];

const STEP_DURATION = 2400;

function FeaturedRunner() {
  const [stepIndex, setStepIndex] = useState(0);
  const [isRunning, setIsRunning] = useState(true);
  const [runVersion, setRunVersion] = useState(0);

  useEffect(() => {
    if (!isRunning) return;

    const timer = window.setInterval(() => {
      setStepIndex((current) => (current + 1) % HUB_STEPS.length);
    }, STEP_DURATION);

    return () => window.clearInterval(timer);
  }, [isRunning]);

  const currentStep = HUB_STEPS[stepIndex];
  const progress = ((stepIndex + 1) / HUB_STEPS.length) * 100;

  const restartFlow = () => {
    setStepIndex(0);
    setRunVersion((current) => current + 1);
    setIsRunning(true);
  };

  return (
    <div className="surface-panel-strong overflow-hidden rounded-[2rem] p-5 sm:p-7 lg:p-8">
      <div className="flex flex-col gap-5 border-b border-black/6 pb-5 lg:flex-row lg:items-end lg:justify-between">
        <div className="max-w-[42rem]">
          <span className="section-kicker">旗舰流程</span>
          <h2 className="mt-5 text-[clamp(2.2rem,4vw,3.4rem)] font-semibold leading-[1.05] text-primary-text">
            一段上下文，如何在系统里长成一份可继续工作的输出。
          </h2>
          <p className="mt-4 text-[1rem] leading-7 text-secondary-text sm:text-[1.06rem]">
            工作现场先被接住，再被收成证据包与上下文胶囊，最后长成一份能继续推进工作的候选输出。
          </p>
        </div>

        <div className="flex flex-col gap-3 sm:flex-row">
          <Button
            size="lg"
            onClick={() => setIsRunning((current) => !current)}
            className="h-12 rounded-2xl bg-primary-text px-5 text-base font-semibold text-white hover:bg-primary-text/94"
          >
            {isRunning ? "暂停演示" : "继续演示"}
          </Button>
          <Button
            size="lg"
            variant="outline"
            onClick={restartFlow}
            className="h-12 rounded-2xl border-black/8 bg-white/82 px-5 text-base font-medium text-primary-text hover:bg-white"
          >
            重新播放
          </Button>
        </div>
      </div>

      <div className="mt-5 flex items-center justify-between gap-4 rounded-[1.4rem] border border-black/6 bg-white/72 px-4 py-3">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
            演示进度
          </div>
          <div className="mt-1 text-sm font-medium text-primary-text">
            当前正在推进第 {stepIndex + 1} 段，证据会继续向右侧输出展开。
          </div>
        </div>
        <div className="hidden sm:flex items-center gap-2 rounded-full border border-sage-green/20 bg-white/84 px-3 py-1.5 text-[11px] font-medium text-secondary-text">
          <span className="inline-flex h-2.5 w-2.5 rounded-full bg-sage-green ambient-pulse" />
          回合 #{runVersion + 1}
        </div>
      </div>

      <div className="mt-4 overflow-hidden rounded-full border border-black/6 bg-white/72 px-1.5 py-1.5">
        <div className="relative h-2.5 overflow-hidden rounded-full bg-black/5">
          <div
            className="absolute inset-y-0 left-0 rounded-full bg-[linear-gradient(90deg,rgba(123,158,172,0.95),rgba(184,169,201,0.86),rgba(156,175,136,0.92))] transition-[width] duration-500"
            style={{ width: `${progress}%` }}
          />
          <div
            className="absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full border border-white bg-primary-text shadow-[0_10px_26px_rgba(45,42,38,0.22)] transition-[left] duration-500"
            style={{ left: `calc(${progress}% - 0.5rem)` }}
          />
        </div>
      </div>

      <div className="mt-5 grid gap-3 lg:grid-cols-4">
        {HUB_STEPS.map((step, index) => (
          <button
            key={step.id}
            type="button"
            onClick={() => {
              setStepIndex(index);
              setIsRunning(false);
            }}
            className={`rounded-full border px-4 py-3 transition-all duration-300 ${
              index === stepIndex
                ? "border-mist-blue/35 bg-white text-primary-text shadow-[0_12px_30px_rgba(45,42,38,0.08)]"
                : "border-black/6 bg-white/55 text-secondary-text/76 hover:bg-white/78"
            }`}
          >
            <div className="text-[10px] font-semibold uppercase tracking-[0.18em] text-secondary-text/64">
              0{index + 1}
            </div>
            <div className="mt-1 text-sm font-medium">{step.phase}</div>
          </button>
        ))}
      </div>

      <div className="mt-6" key={runVersion}>
        <ContextFlowVisual
          stepIndex={stepIndex}
          sceneTitle="把一个普通选题写出锋利感的 7 个方法"
          sceneDescription={currentStep.sceneDescription}
          sceneSignals={currentStep.sceneSignals}
          capsuleTitle={currentStep.capsuleTitle}
          capsuleDescription={currentStep.capsuleDescription}
          capsuleItems={currentStep.raw}
          outputTitle="候选输出"
          outputDescription={currentStep.outputDescription}
          outputItems={currentStep.candidate}
          variant="feature"
        />
      </div>

      <div className="mt-5 grid gap-5 lg:grid-cols-[0.96fr_1.04fr]">
        <div className="surface-panel rounded-[1.7rem] p-5">
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
            当前阶段
          </div>
          <div className="mt-2 text-[1.35rem] font-semibold leading-8 text-primary-text">
            {currentStep.title}
          </div>
          <p className="mt-3 text-sm leading-7 text-secondary-text">{currentStep.note}</p>

          <div className="mt-4 rounded-[1.3rem] border border-black/6 bg-white/82 p-4">
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
              候选响应
            </div>
            <div className="mt-3 text-sm leading-7 text-primary-text">
              {currentStep.assistantLine ?? "上下文正在继续聚拢，系统会把这段经历完整收成一个可继续引用的工作单元。"}
            </div>
          </div>
        </div>

        <div className="surface-panel rounded-[1.7rem] p-5">
          <div className="flex items-center justify-between">
            <div>
              <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                当前入链证据
              </div>
              <div className="mt-2 text-lg font-semibold text-primary-text">当前写入本地的证据</div>
            </div>
            <span className="rounded-full border border-black/6 bg-white/82 px-3 py-1.5 text-[11px] font-medium text-secondary-text">
              事实来源
            </span>
          </div>

          <div className="mt-4 grid gap-2.5 sm:grid-cols-2">
            {currentStep.raw.map((item) => (
              <div
                key={item}
                className="rounded-2xl border border-black/6 bg-white/80 px-4 py-3 text-sm text-primary-text shadow-[0_12px_26px_rgba(45,42,38,0.04)]"
              >
                {item}
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

export default function DemoHubPage() {
  const flagshipRef = useRef<HTMLElement | null>(null);
  const [launchCount, setLaunchCount] = useState(0);

  const launchFlagship = () => {
    setLaunchCount((current) => current + 1);
    flagshipRef.current?.scrollIntoView({ behavior: "smooth", block: "start" });
  };

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-14 pt-12 sm:pb-18 lg:pb-20 lg:pt-18">
          <Container>
            <div className="grid gap-10 lg:grid-cols-[0.9fr_1.1fr] lg:items-center">
              <div className="max-w-[38rem]">
                <span className="section-kicker">Demo Hub</span>
                <h1 className="mt-6 text-[clamp(2.8rem,6vw,5rem)] font-semibold leading-[0.98] text-primary-text">
                  每个入口都从真实片段启动，
                  <span className="mt-3 block text-mist-blue">
                    然后顺着证据链向前展开。
                  </span>
                </h1>
                <p className="mt-6 text-[1.05rem] leading-8 text-secondary-text sm:text-[1.12rem]">
                  Pulseclaw 把阅读、调试、切换、回看这些本来就已经发生的片段直接接入系统。
                  你会看到真实片段如何被保留、收束，并继续长成下一步输出。
                </p>

                <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                  <div className="inline-flex">
                    <Button
                      size="lg"
                      onClick={launchFlagship}
                      className="h-12 rounded-2xl bg-primary-text px-6 text-base font-semibold text-white hover:bg-primary-text/94"
                    >
                      启动内容场景
                    </Button>
                  </div>
                  <Link href="/demo/debug-bug" className="inline-flex">
                    <Button
                      size="lg"
                      variant="outline"
                      className="h-12 rounded-2xl border-black/8 bg-white/82 px-6 text-base font-medium text-primary-text hover:bg-white"
                    >
                      打开调试回放
                    </Button>
                  </Link>
                </div>
              </div>

              <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
                <div className="grid gap-3 sm:grid-cols-2">
                  {[
                    "真实场景驱动",
                    "上下文先于提示词",
                    "原始证据可回放",
                    "候选帮助受边界约束",
                  ].map((item) => (
                    <div
                      key={item}
                      className="rounded-2xl border border-black/6 bg-white/82 px-4 py-4 text-sm font-medium text-primary-text"
                    >
                      <span className="mr-3 inline-flex h-2.5 w-2.5 rounded-full bg-sage-green ambient-pulse" />
                      {item}
                    </div>
                  ))}
                </div>

                <div className="mt-5 rounded-[1.4rem] border border-black/6 bg-white/78 p-4">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                    产品命题
                  </div>
                  <p className="mt-3 text-sm leading-7 text-secondary-text">
                    刚发生过的上下文，值得被系统直接接住，并继续参与下一步协作。
                  </p>
                </div>
              </div>
            </div>
          </Container>
        </section>

        <section
          id="flagship-demo"
          ref={flagshipRef}
          className="scroll-mt-28 pb-16"
        >
          <Container>
            <FeaturedRunner key={launchCount} />
          </Container>
        </section>

        <section className="pb-20 sm:pb-24">
          <Container>
            <div className="mb-8 max-w-[34rem]">
              <span className="section-kicker">Scenarios</span>
              <h2 className="mt-5 text-[clamp(2rem,4vw,3rem)] font-semibold leading-[1.05] text-primary-text">
                同一个产品宇宙里的两条主线。
              </h2>
              <p className="mt-4 text-[1rem] leading-7 text-secondary-text">
                一条主线把内容场景长成写作输出，一条主线把调试现场还原成可验证的证据链。
              </p>
            </div>

            <div className="grid gap-4 lg:grid-cols-3">
              {SCENARIOS.map((scenario) => (
                <ScenarioCard key={scenario.title} {...scenario} />
              ))}
            </div>
          </Container>
        </section>
      </main>

      <Footer />
    </div>
  );
}
