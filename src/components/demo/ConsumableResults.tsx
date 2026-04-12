"use client";

import { useDemo } from "@/lib/demo-context";
import { getEvidenceUpToTime } from "@/lib/demo-data";
import { getCurrentPhase } from "@/lib/demo-script";

export function ConsumableResults() {
  const { timelinePosition } = useDemo();
  const currentPhase = getCurrentPhase(timelinePosition);
  const observed = getEvidenceUpToTime(timelinePosition);
  const latest = observed.at(-1);

  const cards = [
    {
      label: "可带走的结果",
      title: "一段可回放的调试链路",
      description: "当前现场已经被整理成一条能够回看、核对和继续阅读的原始轨迹。",
      tone: "raw",
    },
    {
      label: "当前验证",
      title: latest?.id === "ev-005" || latest?.id === "ev-006" ? "验证已进入链路" : "验证仍在等待",
      description:
        latest?.id === "ev-005" || latest?.id === "ev-006"
          ? "测试通过作为原始记录进入链路，可信度来自它可以被回放。"
          : "系统仍在收集证据，帮助和验证都还不能越级站到前面。",
      tone: "verified",
    },
    {
      label: "候选帮助",
      title: currentPhase === "replay" ? "可以开始提出候选解释" : "候选层仍被延后显示",
      description:
        currentPhase === "replay"
          ? "现在可以给出围绕错误文本、查阅动作与修补片段的候选说明，但它仍然附着在原始记录上。"
          : "Pulseclaw 先把轨迹收住，帮助会晚于证据出现。",
      tone: "candidate",
    },
  ] as const;

  return (
    <div className="surface-panel h-full rounded-[1.8rem] p-5">
      <div className="border-b border-black/6 pb-4">
        <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
          Consumable results
        </div>
        <h3 className="mt-2 text-xl font-semibold text-primary-text">这条链路现在能带走什么</h3>
        <p className="mt-2 text-sm leading-7 text-secondary-text">
          这里展示的是当前回放窗口已经整理出来的可消费结果，但它们仍然站在原始轨迹之后。
        </p>
      </div>

      <div className="mt-5 grid gap-3">
        {cards.map((card) => (
          <div
            key={card.label}
            className="rounded-[1.35rem] border border-black/6 bg-white/82 p-4"
          >
            <div className="flex items-center justify-between gap-3">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
                {card.label}
              </div>
              <span
                className={
                  card.tone === "raw"
                    ? "rounded-full bg-white px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.16em] text-primary-text"
                    : card.tone === "verified"
                      ? "rounded-full bg-sage-green/10 px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.16em] text-sage-green"
                      : "rounded-full bg-digital-lavender/10 px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.16em] text-digital-lavender"
                }
              >
                {card.tone}
              </span>
            </div>
            <div className="mt-3 text-lg font-semibold text-primary-text">{card.title}</div>
            <p className="mt-3 text-sm leading-7 text-secondary-text">{card.description}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
