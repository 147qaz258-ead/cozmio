"use client";

import { DEMO_SCRIPT, PHASE_CONFIG } from "@/lib/demo-script";
import { MOCK_EVIDENCE } from "@/lib/demo-data";
import { useDemo } from "@/lib/demo-context";
import { cn } from "@/lib/utils";

function formatTime(seconds: number) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

export function TimelinePanel() {
  const {
    activeEvidenceId,
    currentStep,
    setActiveEvidenceId,
    setActiveNodeId,
    setTimelinePosition,
    timelinePosition,
  } = useDemo();

  const observedCount = MOCK_EVIDENCE.filter((record) => record.timestamp <= timelinePosition).length;

  return (
    <div className="surface-panel flex h-full flex-col rounded-[1.8rem] p-5">
      <div className="flex items-start justify-between gap-4 border-b border-black/6 pb-4">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
            Replay Timeline
          </div>
          <h3 className="mt-2 text-xl font-semibold text-primary-text">沿着证据链回到发生现场</h3>
          <p className="mt-2 text-sm leading-7 text-secondary-text">
            每一段都能跳回对应时刻，查看当时的 raw record 与之后出现的候选分析。
          </p>
        </div>
        <div className="rounded-[1.2rem] border border-black/6 bg-white/82 px-4 py-3 text-right">
          <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
            Observed
          </div>
          <div className="mt-1 text-[1.4rem] font-semibold text-primary-text">{observedCount}/6</div>
        </div>
      </div>

      <div className="relative mt-5 flex-1 overflow-hidden">
        <div className="absolute left-[1.1rem] top-2 bottom-2 w-px bg-[linear-gradient(180deg,rgba(123,158,172,0.24),rgba(184,169,201,0.18),rgba(156,175,136,0.24))]" />
        <div className="space-y-3 overflow-y-auto pr-1">
          {DEMO_SCRIPT.map((step, index) => {
            const evidence = MOCK_EVIDENCE.find((record) => record.id === step.evidenceId);
            const phase = PHASE_CONFIG[step.phase];
            const isActive = currentStep === index;
            const isSeen = timelinePosition >= step.timestamp;

            return (
              <button
                key={step.id}
                type="button"
                onClick={() => {
                  setTimelinePosition(step.timestamp);
                  setActiveEvidenceId(step.evidenceId);
                  if (step.action?.type === "highlight_node") {
                    setActiveNodeId(step.action.target);
                  }
                }}
                className={cn(
                  "relative flex w-full items-start gap-3 rounded-[1.35rem] border px-4 py-4 text-left transition-all duration-200",
                  isActive
                    ? "border-mist-blue/24 bg-white text-primary-text shadow-[0_18px_36px_rgba(45,42,38,0.07)]"
                    : "border-black/6 bg-white/70 text-secondary-text hover:bg-white/84",
                  isSeen && !isActive && "text-primary-text"
                )}
              >
                <div className="relative z-10 flex w-8 shrink-0 items-center justify-center pt-1">
                  <span
                    className={cn(
                      "inline-flex h-4 w-4 rounded-full border-2 border-white shadow-[0_0_0_4px_rgba(248,247,244,0.95)]",
                      isActive
                        ? "bg-mist-blue"
                        : isSeen
                          ? "bg-sage-green"
                          : "bg-warm-card"
                    )}
                  />
                </div>

                <div className="min-w-0 flex-1">
                  <div className="flex flex-wrap items-center gap-2">
                    <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                      {formatTime(step.timestamp)}
                    </span>
                    <span
                      className={cn(
                        "rounded-full px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em]",
                        step.phase === "capture"
                          ? "bg-mist-blue/10 text-mist-blue"
                          : step.phase === "record"
                            ? "bg-digital-lavender/10 text-digital-lavender"
                            : "bg-sage-green/12 text-sage-green"
                      )}
                    >
                      {phase.label}
                    </span>
                    {step.isSignificant && (
                      <span className="rounded-full border border-black/6 bg-white/82 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-secondary-text/76">
                        pause point
                      </span>
                    )}
                  </div>

                  <div className="mt-2 text-base font-semibold text-primary-text">{step.label}</div>
                  <div className="mt-1 text-sm leading-6 text-secondary-text">{step.description}</div>
                  {evidence && (
                    <div className="mt-3 rounded-2xl border border-black/6 bg-white/82 px-3 py-2.5 text-xs leading-6 text-secondary-text">
                      <span className="font-medium text-primary-text">{evidence.label}</span>
                      <span className="mx-2 text-secondary-text/44">·</span>
                      {evidence.data.windowTitle}
                    </div>
                  )}
                </div>

                {activeEvidenceId === step.evidenceId && (
                  <div className="rounded-full border border-mist-blue/18 bg-mist-blue/10 px-3 py-1.5 text-[11px] font-medium text-mist-blue">
                    raw focus
                  </div>
                )}
              </button>
            );
          })}
        </div>
      </div>
    </div>
  );
}
