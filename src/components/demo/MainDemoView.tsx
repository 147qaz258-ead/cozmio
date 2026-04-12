"use client";

import { AnimatePresence, motion } from "framer-motion";
import { getEvidenceAtTime, getEvidenceUpToTime, MOCK_EVIDENCE, type EvidenceRecord } from "@/lib/demo-data";
import { getCurrentPhase, getStepAtTime, PHASE_CONFIG } from "@/lib/demo-script";
import { useDemo } from "@/lib/demo-context";
import { cn } from "@/lib/utils";

function formatTime(seconds: number) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

function WindowShell({
  title,
  eyebrow,
  children,
}: {
  title: string;
  eyebrow: string;
  children: React.ReactNode;
}) {
  return (
    <div className="rounded-[1.4rem] border border-black/6 bg-white/88 shadow-[0_22px_46px_rgba(45,42,38,0.07)]">
      <div className="flex items-center gap-2 border-b border-black/6 bg-[linear-gradient(90deg,rgba(123,158,172,0.12),rgba(255,255,255,0.78),rgba(184,169,201,0.08))] px-4 py-3">
        <span className="h-2.5 w-2.5 rounded-full bg-[#F18C7E]" />
        <span className="h-2.5 w-2.5 rounded-full bg-[#E9C46A]" />
        <span className="h-2.5 w-2.5 rounded-full bg-[#7FB069]" />
        <div className="ml-3">
          <div className="text-[10px] font-semibold uppercase tracking-[0.16em] text-secondary-text/62">
            {eyebrow}
          </div>
          <div className="mt-0.5 text-sm font-medium text-primary-text">{title}</div>
        </div>
      </div>
      <div className="p-4 sm:p-5">{children}</div>
    </div>
  );
}

function renderScene(evidence: EvidenceRecord) {
  switch (evidence.id) {
    case "ev-001":
      return (
        <WindowShell eyebrow="editor frame" title="src/components/EventTimeline.tsx">
          <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
            <div className="rounded-[1.2rem] bg-primary-text px-4 py-4 text-white">
              <div className="space-y-2 font-mono text-[0.82rem] leading-7">
                {[
                  "12  export function EventTimeline({ payload }) {",
                  "13    const rows = payload.entries.map((entry) => entry.label);",
                  "14    return <TimelineList rows={rows} />;",
                  "15  }",
                ].map((line, index) => (
                  <div
                    key={line}
                    className={cn(
                      "rounded-lg px-3 py-1.5",
                      index === 1 ? "bg-white/10 text-white" : "text-white/72"
                    )}
                  >
                    {line}
                  </div>
                ))}
              </div>
            </div>

            <div className="rounded-[1.2rem] border border-black/6 bg-warm-card/55 p-4">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                为什么这一刻会被保留
              </div>
              <p className="mt-3 text-sm leading-7 text-secondary-text">
                在崩溃出现前，系统只知道用户正停留在这段代码附近。它不会先推断原因，只先把眼前这帧保住。
              </p>
              <div className="mt-4 rounded-2xl bg-white/80 px-3 py-3 text-sm font-medium text-primary-text">
                焦点窗口、可见片段、时间位置都成为 replay 的原始入口。
              </div>
            </div>
          </div>
        </WindowShell>
      );
    case "ev-002":
      return (
        <WindowShell eyebrow="terminal" title="pnpm dev">
          <div className="rounded-[1.2rem] bg-[#161514] px-4 py-4 text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.04)]">
            <div className="font-mono text-[0.82rem] leading-7 text-white/92">
              <div className="text-white/62">$ pnpm dev</div>
              <div className="mt-2 text-[#F18C7E]">
                {"TypeError: Cannot read properties of undefined (reading 'map')"}
              </div>
              <div className="text-white/62">at EventTimeline (src/components/EventTimeline.tsx:13:18)</div>
              <div className="text-white/62">at DebugReplayPanel (src/components/DebugReplayPanel.tsx:44:7)</div>
            </div>
          </div>

          <div className="mt-4 grid gap-3 sm:grid-cols-2">
            <div className="rounded-[1.2rem] border border-black/6 bg-red-50 px-4 py-4">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-red-600/72">
                Raw error text
              </div>
              <p className="mt-3 text-sm leading-7 text-red-700">
                这是最强的一条事实信号。候选分析之后可以围绕它展开，但不能改写它的字面内容。
              </p>
            </div>
            <div className="rounded-[1.2rem] border border-black/6 bg-white/82 px-4 py-4">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                进入链路后的位置
              </div>
              <p className="mt-3 text-sm leading-7 text-secondary-text">
                时间轴会在这里自动停一下，因为这是后续阅读整条证据链的关键锚点。
              </p>
            </div>
          </div>
        </WindowShell>
      );
    case "ev-003":
      return (
        <WindowShell eyebrow="browser research" title="React docs / conditional rendering">
          <div className="grid gap-3">
            <div className="rounded-[1.2rem] border border-black/6 bg-white/82 px-4 py-4">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                被记录下来的查阅动作
              </div>
              <div className="mt-3 text-[1.1rem] font-semibold text-primary-text">
                Render a fallback before mapping an unknown collection.
              </div>
              <p className="mt-3 text-sm leading-7 text-secondary-text">
                这里记录的是用户去查过什么，而不是系统已经确定问题就一定在这里。
              </p>
            </div>
            <div className="grid gap-3 sm:grid-cols-3">
              {[
                "文档标题进入窗口记录",
                "查阅动作进入时间线",
                "外部资料仍不是事实层",
              ].map((item) => (
                <div
                  key={item}
                  className="rounded-[1.1rem] border border-black/6 bg-warm-card/52 px-3 py-3 text-sm leading-6 text-primary-text"
                >
                  {item}
                </div>
              ))}
            </div>
          </div>
        </WindowShell>
      );
    case "ev-004":
      return (
        <WindowShell eyebrow="patch applied" title="EventTimeline.tsx diff view">
          <div className="grid gap-4 xl:grid-cols-[1.05fr_0.95fr]">
            <div className="rounded-[1.2rem] border border-black/6 bg-white/84 p-4">
              <div className="space-y-3 font-mono text-[0.82rem] leading-7">
                <div className="rounded-lg bg-red-50 px-3 py-2 text-red-700">
                  - const rows = payload.entries.map((entry) =&gt; entry.label)
                </div>
                <div className="rounded-lg bg-sage-green/12 px-3 py-2 text-primary-text">
                  + const rows = payload?.entries ?? []
                </div>
                <div className="rounded-lg bg-sage-green/12 px-3 py-2 text-primary-text">
                  + if (!rows.length) return &lt;EmptyState /&gt;
                </div>
              </div>
            </div>

            <div className="rounded-[1.2rem] border border-black/6 bg-white/82 p-4">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                候选层在这里才获得资格
              </div>
              <p className="mt-3 text-sm leading-7 text-secondary-text">
                到了这一步，系统才有资格挂上“可能是列表缺少保护”这样的候选解释，因为它终于能同时回指错误文本与修补动作。
              </p>
              <div className="mt-4 rounded-2xl border border-digital-lavender/18 bg-digital-lavender/8 px-4 py-3 text-sm leading-7 text-primary-text">
                candidate: payload.entries may be undefined before first render
              </div>
            </div>
          </div>
        </WindowShell>
      );
    case "ev-005":
      return (
        <WindowShell eyebrow="verification" title="pnpm test EventTimeline">
          <div className="rounded-[1.2rem] bg-[#161514] px-4 py-4 text-white">
            <div className="font-mono text-[0.82rem] leading-7 text-white/92">
              <div className="text-white/62">$ pnpm test EventTimeline</div>
              <div className="mt-2 text-[#95B386]">PASS src/components/EventTimeline.test.tsx</div>
              <div className="text-white/62">✓ renders empty state without crashing</div>
              <div className="text-white/62">✓ keeps replay panel responsive</div>
            </div>
          </div>

          <div className="mt-4 rounded-[1.2rem] border border-black/6 bg-white/82 p-4">
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
              为什么这仍然属于原始记录
            </div>
            <p className="mt-3 text-sm leading-7 text-secondary-text">
              “测试通过”看起来像结论，但在这条系统里它依然只是另一条原始记录。真正的可信度来自它能被回放，而不是被一句总结替代。
            </p>
          </div>
        </WindowShell>
      );
    default:
      return (
        <WindowShell eyebrow="replay summary" title="Pulseclaw evidence summary">
          <div className="grid gap-3 sm:grid-cols-3">
            {[
              { value: "6", label: "raw records" },
              { value: "1", label: "replayable chain" },
              { value: "2", label: "bounded candidates" },
            ].map((item) => (
              <div
                key={item.label}
                className="rounded-[1.2rem] border border-black/6 bg-white/82 px-4 py-4 text-center"
              >
                <div className="text-[1.8rem] font-semibold text-primary-text">{item.value}</div>
                <div className="mt-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
                  {item.label}
                </div>
              </div>
            ))}
          </div>
        </WindowShell>
      );
  }
}

export function MainDemoView() {
  const { activeEvidenceId, isPaused, isPlaying, timelinePosition } = useDemo();

  const currentEvidence =
    (activeEvidenceId
      ? MOCK_EVIDENCE.find((record) => record.id === activeEvidenceId)
      : getEvidenceAtTime(timelinePosition)) ?? MOCK_EVIDENCE[0];

  const observedRecords = getEvidenceUpToTime(timelinePosition);
  const currentPhase = getCurrentPhase(timelinePosition);
  const phaseMeta = PHASE_CONFIG[currentPhase];
  const activeStep = getStepAtTime(timelinePosition);

  return (
      <div className="flex h-full flex-col gap-4">
        <div className="flex flex-col gap-4 border-b border-black/6 pb-4 xl:flex-row xl:items-center xl:justify-between">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
            运行主舞台
          </div>
          <h3 className="mt-2 text-[1.8rem] font-semibold leading-tight text-primary-text">
            主舞台始终在展示刚才真实发生的那一刻。
            </h3>
          <p className="mt-3 max-w-[42rem] text-sm leading-7 text-secondary-text">
            {activeStep?.description ?? "系统正在沿着证据链推进。"}
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-2">
          <span className="rounded-full border border-black/6 bg-white/82 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text">
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
          <span className="rounded-full bg-mist-blue/10 px-3 py-2 text-[11px] font-semibold uppercase tracking-[0.16em] text-mist-blue">
            {formatTime(timelinePosition)}
          </span>
        </div>
      </div>

      <div className="grid flex-1 gap-4 xl:grid-cols-[1.08fr_0.92fr]">
        <div className="rounded-[1.8rem] border border-black/6 bg-[linear-gradient(135deg,rgba(123,158,172,0.12),rgba(255,255,255,0.9),rgba(184,169,201,0.12))] p-3 sm:p-4">
          <AnimatePresence mode="wait">
            <motion.div
              key={currentEvidence.id}
              initial={{ opacity: 0, y: 18 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -12 }}
              transition={{ duration: 0.28, ease: "easeOut" }}
            >
              {renderScene(currentEvidence)}
            </motion.div>
          </AnimatePresence>
        </div>

        <div className="grid gap-4">
          <div className="rounded-[1.45rem] border border-black/6 bg-white/84 p-4">
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
              当前原始记录
            </div>
            <div className="mt-3 text-lg font-semibold text-primary-text">{currentEvidence.label}</div>
            <p className="mt-3 text-sm leading-7 text-secondary-text">{currentEvidence.data.note}</p>
          </div>

          <div className="rounded-[1.45rem] border border-black/6 bg-white/84 p-4">
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
              当前阶段
            </div>
            <div className="mt-3 text-lg font-semibold text-primary-text">{phaseMeta.title}</div>
            <p className="mt-2 text-sm leading-7 text-secondary-text">{phaseMeta.description}</p>
          </div>

          <div className="flex-1 rounded-[1.45rem] border border-black/6 bg-white/84 p-4">
            <div className="flex items-center justify-between gap-3">
              <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                已进入链路的记录
              </div>
              <span className="rounded-full border border-black/6 bg-white/84 px-3 py-1.5 text-[11px] font-medium text-secondary-text">
                {observedRecords.length} records
              </span>
            </div>

            <div className="mt-4 space-y-2.5">
              {observedRecords.map((record) => (
                <div
                  key={record.id}
                  className={cn(
                    "flex items-start gap-3 rounded-2xl border px-3 py-3",
                    currentEvidence.id === record.id
                      ? "border-mist-blue/22 bg-mist-blue/8"
                      : "border-black/6 bg-white/74"
                  )}
                >
                  <span className="mt-1 inline-flex h-2.5 w-2.5 rounded-full bg-sage-green ambient-pulse" />
                  <div>
                    <div className="text-sm font-medium text-primary-text">{record.label}</div>
                    <div className="mt-1 text-xs leading-6 text-secondary-text">
                      {record.data.windowTitle}
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>

      <div className="rounded-[1.35rem] border border-black/6 bg-white/84 px-4 py-4 text-sm leading-7 text-secondary-text">
        候选层叠加在这条轨迹之上工作；原始轨迹本身始终保持可回放、可核对、可重新阅读。
      </div>
    </div>
  );
}
