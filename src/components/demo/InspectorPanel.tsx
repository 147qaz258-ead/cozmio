"use client";

import { getEvidenceAtTime, getEvidenceUpToTime, MOCK_EVIDENCE } from "@/lib/demo-data";
import { useDemo } from "@/lib/demo-context";
import { cn } from "@/lib/utils";

interface InspectorPanelProps {
  activeEvidenceId?: string | null;
  onEvidenceClick?: (evidenceId: string) => void;
}

function formatTimestamp(seconds: number) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

export function InspectorPanel({ activeEvidenceId, onEvidenceClick }: InspectorPanelProps) {
  const { setActiveEvidenceId, timelinePosition } = useDemo();

  const currentEvidence = activeEvidenceId
    ? MOCK_EVIDENCE.find((record) => record.id === activeEvidenceId)
    : getEvidenceAtTime(timelinePosition);
  const observedRecords = getEvidenceUpToTime(timelinePosition);

  return (
    <div className="surface-panel h-full rounded-[1.8rem] p-5">
      <div className="flex items-start justify-between gap-4 border-b border-black/6 pb-4">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
            原始检查器
          </div>
          <h3 className="mt-2 text-xl font-semibold text-primary-text">当前查看的原始记录</h3>
          <p className="mt-2 text-sm leading-7 text-secondary-text">
            每条记录都带着自己的时间、窗口和来源，随时可以沿着时间线逐条核对。
          </p>
        </div>
        <span className="rounded-full border border-black/6 bg-white/82 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text">
          事实来源
        </span>
      </div>

      {currentEvidence ? (
        <div className="mt-5 grid gap-4 xl:grid-cols-[1.05fr_0.95fr]">
          <div className="space-y-4">
            <div className="rounded-[1.35rem] border border-black/6 bg-white/82 p-4">
              <div className="flex flex-wrap items-center gap-2">
                <span className="rounded-full bg-mist-blue/10 px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-mist-blue">
                  {currentEvidence.type}
                </span>
                <span className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                  {formatTimestamp(currentEvidence.timestamp)}
                </span>
              </div>
              <h4 className="mt-3 text-lg font-semibold text-primary-text">{currentEvidence.label}</h4>
              <p className="mt-3 text-sm leading-7 text-secondary-text">{currentEvidence.data.content}</p>
            </div>

            <div className="grid gap-3 sm:grid-cols-2">
              <div className="rounded-[1.25rem] border border-black/6 bg-white/78 p-4">
                <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                  Window
                </div>
                <div className="mt-2 text-sm font-medium text-primary-text">
                  {currentEvidence.data.windowTitle ?? "Unknown window"}
                </div>
              </div>
              <div className="rounded-[1.25rem] border border-black/6 bg-white/78 p-4">
                <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                  Process
                </div>
                <div className="mt-2 text-sm font-medium text-primary-text">
                  {currentEvidence.data.processName ?? "Unknown process"}
                </div>
              </div>
              <div className="rounded-[1.25rem] border border-black/6 bg-white/78 p-4">
                <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                  File / Command
                </div>
                <div className="mt-2 text-sm font-medium text-primary-text">
                  {currentEvidence.data.filePath ?? currentEvidence.data.command ?? "No file path attached"}
                </div>
              </div>
              <div className="rounded-[1.25rem] border border-black/6 bg-white/78 p-4">
                <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                  Evidence ID
                </div>
                <div className="mt-2 font-mono text-sm text-primary-text">{currentEvidence.id}</div>
              </div>
            </div>

            {(currentEvidence.data.errorText || currentEvidence.data.snippet) && (
              <div className="rounded-[1.35rem] border border-black/6 bg-primary-text p-4 text-white shadow-[0_22px_40px_rgba(45,42,38,0.12)]">
                <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-white/58">
                  Raw excerpt
                </div>
                <pre className="mt-3 whitespace-pre-wrap break-words font-mono text-[0.82rem] leading-7 text-white/92">
                  {currentEvidence.data.errorText ?? currentEvidence.data.snippet}
                </pre>
              </div>
            )}
          </div>

          <div className="flex h-full flex-col gap-4">
            <div className="rounded-[1.35rem] border border-black/6 bg-white/78 p-4">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                Why it matters
              </div>
              <p className="mt-3 text-sm leading-7 text-secondary-text">
                {currentEvidence.data.note ??
                  "这条记录说明此刻真实发生了什么，但它本身不替你完成推理。"}
              </p>
            </div>

            <div className="flex-1 rounded-[1.35rem] border border-black/6 bg-white/78 p-4">
              <div className="flex items-center justify-between gap-3">
                <div>
                  <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                    Observed so far
                  </div>
                  <div className="mt-1 text-sm font-medium text-primary-text">
                    已经进入当前回放窗口的记录
                  </div>
                </div>
                <span className="rounded-full border border-black/6 bg-white/80 px-3 py-1.5 text-[11px] font-medium text-secondary-text">
                  {observedRecords.length} records
                </span>
              </div>

              <div className="mt-4 space-y-2.5">
                {observedRecords.map((record) => (
                  <button
                    key={record.id}
                    type="button"
                    onClick={() => {
                      setActiveEvidenceId(record.id);
                      onEvidenceClick?.(record.id);
                    }}
                    className={cn(
                      "flex w-full items-start justify-between gap-3 rounded-2xl border px-3 py-3 text-left transition-colors",
                      currentEvidence.id === record.id
                        ? "border-mist-blue/22 bg-mist-blue/8"
                        : "border-black/6 bg-white/74 hover:bg-white"
                    )}
                  >
                    <div>
                      <div className="text-sm font-medium text-primary-text">{record.label}</div>
                      <div className="mt-1 text-xs leading-6 text-secondary-text">
                        {record.data.windowTitle}
                      </div>
                    </div>
                    <span className="shrink-0 text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
                      {formatTimestamp(record.timestamp)}
                    </span>
                  </button>
                ))}
              </div>
            </div>
          </div>
        </div>
      ) : (
        <div className="flex min-h-[16rem] items-center justify-center text-sm text-secondary-text">
          当前时间点还没有可展示的 evidence record。
        </div>
      )}
    </div>
  );
}
