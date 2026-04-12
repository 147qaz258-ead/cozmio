"use client";

import { ChangeEvent } from "react";
import { DemoMode, useDemo } from "@/lib/demo-context";
import { getCurrentPhase, PHASE_CONFIG } from "@/lib/demo-script";
import { cn } from "@/lib/utils";

const modeLabels: Record<DemoMode, string> = {
  autoplay: "自动回放",
  "step-by-step": "逐步取证",
  "explore-freely": "自由探索",
};

function formatTime(seconds: number) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

export function ReplayControls() {
  const {
    currentStep,
    mode,
    nextStep,
    pause,
    play,
    prevStep,
    reset,
    resume,
    setTimelinePosition,
    timelinePosition,
    isPaused,
    isPlaying,
  } = useDemo();

  const currentPhase = getCurrentPhase(timelinePosition);
  const phaseMeta = PHASE_CONFIG[currentPhase];

  const handleSeek = (event: ChangeEvent<HTMLInputElement>) => {
    setTimelinePosition(Number(event.target.value));
  };

  const handlePrimaryAction = () => {
    if (mode === "autoplay") {
      if (isPlaying && !isPaused) {
        pause();
        return;
      }
      if (isPaused) {
        resume();
        return;
      }
      play();
      return;
    }

    if (mode === "step-by-step") {
      nextStep();
      return;
    }

    play();
  };

  const primaryLabel =
    mode === "autoplay"
      ? isPlaying && !isPaused
        ? "暂停演示"
        : isPaused
          ? "继续演示"
          : "开始演示"
      : mode === "step-by-step"
        ? "下一段"
        : "切回自动模式";

  const replayProgress = Math.min((timelinePosition / 50) * 100, 100);

  return (
    <div className="surface-panel rounded-[1.65rem] p-4 sm:p-5">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:justify-between">
        <div className="flex flex-wrap items-center gap-2.5">
          <button
            type="button"
            onClick={reset}
            className="rounded-full border border-black/7 bg-white/78 px-4 py-2 text-sm font-medium text-primary-text transition-colors hover:bg-white"
          >
            从头开始
          </button>
          <button
            type="button"
            onClick={prevStep}
            className="rounded-full border border-black/7 bg-white/68 px-4 py-2 text-sm font-medium text-secondary-text transition-colors hover:bg-white hover:text-primary-text"
          >
            上一段
          </button>
          <button
            type="button"
            onClick={handlePrimaryAction}
            className="rounded-full bg-primary-text px-5 py-2.5 text-sm font-semibold text-white shadow-[0_16px_30px_rgba(45,42,38,0.15)] transition-colors hover:bg-primary-text/94"
          >
            {primaryLabel}
          </button>
          <button
            type="button"
            onClick={() => {
              if (mode === "step-by-step") {
                nextStep();
                return;
              }
              setTimelinePosition(timelinePosition + 5);
            }}
            className="rounded-full border border-black/7 bg-white/68 px-4 py-2 text-sm font-medium text-secondary-text transition-colors hover:bg-white hover:text-primary-text"
          >
            {mode === "step-by-step" ? "继续下一段" : "前进 5s"}
          </button>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <span className="section-kicker !px-3 !py-2 !text-[10px]">模式 · {modeLabels[mode]}</span>
          <span className="rounded-full border border-mist-blue/16 bg-white/78 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text">
            {phaseMeta.label}
          </span>
          <span
            className={cn(
              "rounded-full px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.16em]",
              isPlaying && !isPaused
                ? "bg-sage-green/12 text-sage-green"
                : "bg-black/5 text-secondary-text"
            )}
          >
            {isPlaying && !isPaused ? "运行中" : "已暂停"}
          </span>
        </div>
      </div>

      <div className="mt-5 rounded-[1.4rem] border border-black/6 bg-white/78 p-4">
        <div className="flex flex-col gap-4 lg:flex-row lg:items-center lg:justify-between">
          <div>
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
              时间游标
            </div>
            <div className="mt-2 text-sm leading-7 text-secondary-text">
              当前位于第 <span className="font-semibold text-primary-text">{currentStep + 1}</span> 段，
              正在展示 <span className="font-semibold text-primary-text">{phaseMeta.title}</span> 阶段。
            </div>
          </div>

          <div className="text-right">
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
              回放窗口
            </div>
            <div className="mt-2 text-[1.35rem] font-semibold text-primary-text">
              {formatTime(timelinePosition)}
              <span className="ml-2 text-sm font-medium text-secondary-text">/ 00:50</span>
            </div>
          </div>
        </div>

        <div className="mt-5">
          <div className="mb-4 overflow-hidden rounded-full border border-black/6 bg-white/88 px-1.5 py-1.5">
            <div className="relative h-2.5 overflow-hidden rounded-full bg-black/5">
              <div
                className="absolute inset-y-0 left-0 rounded-full bg-[linear-gradient(90deg,rgba(123,158,172,0.95),rgba(184,169,201,0.86),rgba(156,175,136,0.92))] transition-[width] duration-200"
                style={{ width: `${replayProgress}%` }}
              />
              <div
                className={cn(
                  "absolute top-1/2 h-4 w-4 -translate-y-1/2 rounded-full border border-white bg-primary-text shadow-[0_8px_20px_rgba(45,42,38,0.2)] transition-[left] duration-200",
                  isPlaying && !isPaused && "ambient-pulse"
                )}
                style={{ left: `calc(${replayProgress}% - 0.5rem)` }}
              />
            </div>
          </div>

          <input
            type="range"
            min="0"
            max="50"
            step="1"
            value={timelinePosition}
            onChange={handleSeek}
            className="w-full cursor-pointer appearance-none bg-transparent [&::-webkit-slider-runnable-track]:h-2 [&::-webkit-slider-runnable-track]:rounded-full [&::-webkit-slider-runnable-track]:bg-[linear-gradient(90deg,rgba(123,158,172,0.2),rgba(184,169,201,0.2),rgba(156,175,136,0.2))] [&::-webkit-slider-thumb]:-mt-[5px] [&::-webkit-slider-thumb]:size-4 [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:border [&::-webkit-slider-thumb]:border-white [&::-webkit-slider-thumb]:bg-primary-text [&::-webkit-slider-thumb]:shadow-[0_8px_20px_rgba(45,42,38,0.2)]"
          />

          <div className="mt-3 grid gap-2 sm:grid-cols-3">
            {Object.entries(PHASE_CONFIG).map(([phaseKey, phase]) => (
              <div
                key={phaseKey}
                className={cn(
                  "rounded-2xl border px-3 py-3 text-sm transition-colors",
                  currentPhase === phaseKey
                    ? "border-mist-blue/18 bg-mist-blue/8 text-primary-text"
                    : "border-black/6 bg-white/65 text-secondary-text"
                )}
              >
                <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                  {phase.label}
                </div>
                <div className="mt-1 font-medium">{phase.title}</div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
