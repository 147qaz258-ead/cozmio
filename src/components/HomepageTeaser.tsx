"use client";

import { useEffect, useState } from "react";
import { ContextFlowVisual } from "@/components/demo/ContextFlowVisual";
import { cn } from "@/lib/utils";

type StoryStep = {
  id: string;
  phase: string;
  label: string;
  note: string;
  sceneDescription: string;
  sceneSignals: string[];
  capsuleTitle: string;
  capsuleDescription: string;
  capsuleItems: string[];
  outputTitle: string;
  outputDescription: string;
  outputItems: string[];
};

const STORY_STEPS: StoryStep[] = [
  {
    id: "observe",
    phase: "Observe",
    label: "真实阅读片段",
    note: "工作里的上下文本来就已经在流动。",
    sceneDescription: "用户来回看标题、封面与导语，随后切回自己的灵感库。",
    sceneSignals: ["重复查看标题", "封面区域停留", "切回灵感库"],
    capsuleTitle: "信号开始聚拢",
    capsuleDescription: "窗口、关键帧与停留动作准备收束成一段可回放记录。",
    capsuleItems: ["window", "frame", "dwell", "switch"],
    outputTitle: "候选输出",
    outputDescription: "帮助还没有出现，系统先把上下文保留下来。",
    outputItems: ["候选帮助尚未出现", "系统仍在积累原始证据", "帮助会晚于证据"],
  },
  {
    id: "preserve",
    phase: "Preserve",
    label: "证据包开始成形",
    note: "原始信号先变成证据包，再进入下一层。",
    sceneDescription: "时间、窗口顺序与关键帧被写入本地证据链，随时可以回放。",
    sceneSignals: ["浏览器文章页聚焦", "回看行为被保留", "回到创作工作台"],
    capsuleTitle: "保留的上下文",
    capsuleDescription: "同一段经历开始具备稳定的引用关系与时间边界。",
    capsuleItems: ["append-only", "local-first", "replayable", "time-bound"],
    outputTitle: "候选输出",
    outputDescription: "候选层还在等待，原始证据先站到舞台中央。",
    outputItems: ["候选层暂未打开", "原始证据已可回放", "来源关系正在收束"],
  },
  {
    id: "condense",
    phase: "Condense",
    label: "上下文胶囊形成",
    note: "系统可以开始组织上下文，但还不能越级替用户下结论。",
    sceneDescription: "这段阅读痕迹开始表达出更清晰的关注点：标题力度、封面气质与结构节奏。",
    sceneSignals: ["标题力度浮现", "封面气质被标记", "结构节奏被提取"],
    capsuleTitle: "Context Capsule",
    capsuleDescription: "证据包被整理成一个可引用、可回放、可验证的上下文单元。",
    capsuleItems: ["标题力度", "封面气质", "结构节奏", "可回放依据"],
    outputTitle: "候选输出",
    outputDescription: "候选帮助开始露面，但仍然受证据链约束。",
    outputItems: ["标题力度拆解", "封面气质关键词", "结构节奏笔记", "一版候选提纲"],
  },
  {
    id: "assist",
    phase: "Assist",
    label: "候选帮助出现",
    note: "帮助顺着证据长出来，而不是突然闯进来。",
    sceneDescription: "这时 AI 才有资格把上下文转成一份可继续工作的候选输出。",
    sceneSignals: ["证据链已经闭合", "上下文胶囊已形成", "帮助开始展开"],
    capsuleTitle: "Ready to Assist",
    capsuleDescription: "所有输出仍然保持可回指、可撤回、可重新核对。",
    capsuleItems: ["traceable", "bounded", "candidate", "ready"],
    outputTitle: "候选输出",
    outputDescription: "现在出现的是一组可以直接继续工作的候选结果。",
    outputItems: ["公众号写作 brief", "三种标题语气候选", "封面关键词建议", "结构节奏提醒"],
  },
];

const STEP_DURATION = 2200;

function StepPill({
  active,
  step,
  index,
}: {
  active: boolean;
  step: StoryStep;
  index: number;
}) {
  return (
    <div
      className={cn(
        "rounded-full border px-3 py-2 text-left transition-all duration-500",
        active
          ? "border-mist-blue/40 bg-white text-primary-text shadow-[0_12px_30px_rgba(45,42,38,0.08)]"
          : "border-black/6 bg-white/50 text-secondary-text/82"
      )}
    >
      <div className="text-[10px] font-semibold uppercase tracking-[0.18em] text-secondary-text/72">
        0{index + 1}
      </div>
      <div className="mt-1 text-xs font-medium">{step.phase}</div>
    </div>
  );
}

export function HomepageTeaser() {
  const [stepIndex, setStepIndex] = useState(0);

  useEffect(() => {
    const timer = window.setInterval(() => {
      setStepIndex((current) => (current + 1) % STORY_STEPS.length);
    }, STEP_DURATION);

    return () => window.clearInterval(timer);
  }, []);

  const currentStep = STORY_STEPS[stepIndex];

  return (
    <div className="surface-panel-strong relative overflow-hidden rounded-[2rem] p-4 sm:p-5">
      <div className="pointer-events-none absolute inset-0">
        <div className="absolute inset-x-0 top-0 h-28 bg-gradient-to-b from-white/72 to-transparent" />
        <div className="absolute right-[-12%] top-[-10%] h-56 w-56 rounded-full bg-mist-blue/14 blur-3xl" />
        <div className="absolute bottom-[-18%] left-[-8%] h-60 w-60 rounded-full bg-digital-lavender/14 blur-3xl" />
      </div>

      <div className="relative">
        <div className="story-divider flex items-center justify-between pb-4">
          <div>
            <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/76">
              Pulseclaw Runtime
            </div>
            <div className="mt-1 text-sm font-medium text-primary-text">
              上下文在系统里逐步聚拢，再长成帮助
            </div>
          </div>

          <div className="rounded-full border border-sage-green/20 bg-white/72 px-3 py-1.5 text-[11px] font-medium text-secondary-text shadow-[0_10px_20px_rgba(45,42,38,0.05)]">
            <span className="mr-2 inline-flex h-2 w-2 rounded-full bg-sage-green ambient-pulse" />
            runtime alive
          </div>
        </div>

        <div className="mt-4 grid gap-3 sm:grid-cols-4">
          {STORY_STEPS.map((step, index) => (
            <StepPill
              key={step.id}
              active={stepIndex === index}
              step={step}
              index={index}
            />
          ))}
        </div>

        <div className="mt-5">
          <ContextFlowVisual
            stepIndex={stepIndex}
            sceneTitle="把一个普通选题写出锋利感的 7 个方法"
            sceneDescription={currentStep.sceneDescription}
            sceneSignals={currentStep.sceneSignals}
            capsuleTitle={currentStep.capsuleTitle}
            capsuleDescription={currentStep.capsuleDescription}
            capsuleItems={currentStep.capsuleItems}
            outputTitle={currentStep.outputTitle}
            outputDescription={currentStep.outputDescription}
            outputItems={currentStep.outputItems}
            variant="compact"
          />
        </div>

        <div className="story-divider mt-5 flex flex-wrap items-center justify-between gap-3 pt-4">
          <div className="text-sm text-secondary-text">{currentStep.note}</div>
          <div className="flex items-center gap-2 rounded-full border border-black/6 bg-white/72 px-3 py-2 text-[11px] font-medium text-secondary-text">
            <span className="inline-flex h-2 w-2 rounded-full bg-mist-blue ambient-pulse" />
            信号仍在持续汇聚
          </div>
        </div>
      </div>
    </div>
  );
}
