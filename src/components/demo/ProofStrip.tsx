"use client";

import { useState } from "react";
import { cn } from "@/lib/utils";

interface ProofBadge {
  id: string;
  title: string;
  description: string;
  detail: string;
}

const proofBadges: ProofBadge[] = [
  {
    id: "raw-truth",
    title: "Raw is truth",
    description: "原始帧、终端文本和窗口切换才是真相层。",
    detail: "任何总结、候选解释或推荐都必须回到这些记录上核对。",
  },
  {
    id: "append-only",
    title: "Append-only",
    description: "记录一旦写入，就不会被候选层改写。",
    detail: "这保证了 replay 时看到的仍然是当时留下来的证据，而不是后来修饰过的故事。",
  },
  {
    id: "local-first",
    title: "Local-first",
    description: "上下文首先停留在本地，不靠云端存活。",
    detail: "端侧保留与回放是这条链路成立的前提，模型解释是叠加层，不是基础设施。",
  },
  {
    id: "candidate-bounded",
    title: "Candidate bounded",
    description: "Graph 里的派生层和候选层必须被降权显示。",
    detail: "它们可以帮助理解，但不能替代证据本身，也不能越级成为最终裁决。",
  },
  {
    id: "replayable",
    title: "Replayable",
    description: "每段帮助都能回跳到发生现场。",
    detail: "时间轴、检查器与 graph 联动的意义，就是让帮助始终能回指到证据链。",
  },
];

interface ProofStripProps {
  onBadgeClick?: (badgeId: string) => void;
}

export function ProofStrip({ onBadgeClick }: ProofStripProps) {
  const [activeBadge, setActiveBadge] = useState<string>("raw-truth");
  const selectedBadge = proofBadges.find((badge) => badge.id === activeBadge) ?? proofBadges[0];

  return (
    <div className="surface-panel rounded-[1.85rem] p-5">
      <div className="flex flex-col gap-4 lg:flex-row lg:items-end lg:justify-between">
        <div className="max-w-[36rem]">
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
            系统原则
          </div>
          <h3 className="mt-2 text-xl font-semibold text-primary-text">Pulseclaw 的五条系统约束</h3>
          <p className="mt-2 text-sm leading-7 text-secondary-text">
            原始记录、回放能力、端侧保留和候选层边界，共同构成这套系统的可信度。
          </p>
        </div>

        <div className="rounded-[1.25rem] border border-black/6 bg-white/82 px-4 py-3 text-sm leading-7 text-secondary-text lg:max-w-[29rem]">
          <span className="font-semibold text-primary-text">{selectedBadge.title}</span>
          <span className="mx-2 text-secondary-text/40">·</span>
          {selectedBadge.detail}
        </div>
      </div>

      <div className="mt-5 grid gap-3 lg:grid-cols-5">
        {proofBadges.map((badge) => (
          <button
            key={badge.id}
            type="button"
            onClick={() => {
              setActiveBadge(badge.id);
              onBadgeClick?.(badge.id);
            }}
            className={cn(
              "rounded-[1.3rem] border px-4 py-4 text-left transition-all duration-200",
              activeBadge === badge.id
                ? "border-mist-blue/22 bg-white text-primary-text shadow-[0_18px_36px_rgba(45,42,38,0.07)]"
                : "border-black/6 bg-white/70 text-secondary-text hover:bg-white/84"
            )}
          >
            <div className="text-sm font-semibold">{badge.title}</div>
            <div className="mt-2 text-sm leading-7">{badge.description}</div>
          </button>
        ))}
      </div>
    </div>
  );
}
