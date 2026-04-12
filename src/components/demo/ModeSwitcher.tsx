"use client";

import { DemoMode, useDemo } from "@/lib/demo-context";
import { cn } from "@/lib/utils";

const MODES: { id: DemoMode; label: string; description: string; shortcut: string }[] = [
  {
    id: "autoplay",
    label: "自动回放",
    description: "让整条证据链自己推进，默认展示系统如何观察、保留、回放。",
    shortcut: "Space",
  },
  {
    id: "step-by-step",
    label: "逐步取证",
    description: "按阶段停留，每次只看一段关键片段和它对应的原始记录。",
    shortcut: "← / →",
  },
  {
    id: "explore-freely",
    label: "自由探索",
    description: "手动拖动时间轴与面板，验证 replay 与 raw/candidate 的边界。",
    shortcut: "← / →",
  },
];

export function ModeSwitcher() {
  const { mode, setMode } = useDemo();
  const activeMode = MODES.find((entry) => entry.id === mode) ?? MODES[0];

  return (
    <div className="surface-panel rounded-[1.5rem] p-2.5">
      <div className="flex flex-wrap gap-2">
        {MODES.map((entry) => (
          <button
            key={entry.id}
            type="button"
            onClick={() => setMode(entry.id)}
            aria-pressed={mode === entry.id}
            className={cn(
              "flex min-w-[10.5rem] flex-1 items-start justify-between gap-3 rounded-[1.2rem] border px-4 py-3 text-left transition-all duration-200",
              mode === entry.id
                ? "border-mist-blue/24 bg-white text-primary-text shadow-[0_16px_32px_rgba(45,42,38,0.07)]"
                : "border-black/6 bg-white/64 text-secondary-text hover:bg-white/84"
            )}
          >
            <div>
              <div className="text-sm font-semibold">{entry.label}</div>
              <div className="mt-1 text-xs leading-6">{entry.description}</div>
            </div>
            <span
              className={cn(
                "rounded-full border px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em]",
                mode === entry.id
                  ? "border-mist-blue/22 bg-mist-blue/10 text-mist-blue"
                  : "border-black/6 bg-white/70 text-secondary-text/70"
              )}
            >
              {entry.shortcut}
            </span>
          </button>
        ))}
      </div>

      <div className="mt-3 rounded-[1.2rem] border border-black/6 bg-white/78 px-4 py-3 text-sm leading-7 text-secondary-text">
        当前模式会优先呈现 <span className="font-medium text-primary-text">{activeMode.label}</span> 的交互节奏。
        原始证据始终保持不变，变化的是你查看这条链路的方式。
      </div>
    </div>
  );
}
