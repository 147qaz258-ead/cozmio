"use client";

import Link from "next/link";
import { DemoProvider, useDemo } from "@/lib/demo-context";
import { MOCK_EVIDENCE } from "@/lib/demo-data";
import { MOCK_GRAPH_NODES } from "@/lib/graph-data";
import { getCurrentPhase, PHASE_CONFIG } from "@/lib/demo-script";
import { Container } from "@/components/layout/Container";
import { Footer } from "@/components/layout/Footer";
import { Header } from "@/components/layout/Header";
import { MainDemoView } from "@/components/demo/MainDemoView";
import { ModeSwitcher } from "@/components/demo/ModeSwitcher";
import { TimelinePanel } from "@/components/demo/TimelinePanel";
import { ReplayControls } from "@/components/demo/ReplayControls";
import { InspectorPanel } from "@/components/demo/InspectorPanel";
import { GraphView } from "@/components/demo/GraphView";
import { ProofStrip } from "@/components/demo/ProofStrip";
import { ConsumableResults } from "@/components/demo/ConsumableResults";
import { SourceTraceDrawer } from "@/components/demo/SourceTraceDrawer";
import { Button } from "@/components/ui/button";
import { useState } from "react";

function formatTime(seconds: number) {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}

function DebugBugProofContent() {
  const {
    activeEvidenceId,
    activeNodeId,
    isPaused,
    isPlaying,
    mode,
    reset,
    setActiveEvidenceId,
    setActiveNodeId,
    setTimelinePosition,
    timelinePosition,
  } = useDemo();

  const [selectedDerivedId, setSelectedDerivedId] = useState<string | null>(null);

  const observedCount = MOCK_EVIDENCE.filter((record) => record.timestamp <= timelinePosition).length;
  const currentPhase = getCurrentPhase(timelinePosition);
  const phaseMeta = PHASE_CONFIG[currentPhase];

  const handleNodeClick = (nodeId: string) => {
    setSelectedDerivedId(nodeId);
    setActiveNodeId(nodeId);
    const node = MOCK_GRAPH_NODES.find((entry) => entry.id === nodeId);
    const firstEvidence = node?.evidenceRefs[0];
    if (firstEvidence) {
      const matchingRecord = MOCK_EVIDENCE.find((record) => record.id === firstEvidence);
      if (matchingRecord) {
        setActiveEvidenceId(matchingRecord.id);
        setTimelinePosition(matchingRecord.timestamp);
      }
    }
  };

  const handleEvidenceClick = (evidenceId: string) => {
    const record = MOCK_EVIDENCE.find((entry) => entry.id === evidenceId);
    if (!record) return;

    setActiveEvidenceId(evidenceId);
    setTimelinePosition(record.timestamp);
  };

  const handleProofBadgeClick = (badgeId: string) => {
    if (badgeId === "raw-truth") {
      handleEvidenceClick("ev-002");
      return;
    }
    if (badgeId === "append-only") {
      handleEvidenceClick("ev-006");
      return;
    }
    if (badgeId === "local-first") {
      handleEvidenceClick("ev-001");
      return;
    }
    if (badgeId === "candidate-bounded") {
      handleNodeClick("candidate-guard");
      return;
    }
    handleEvidenceClick("ev-005");
  };

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-10 pt-12 sm:pb-12 lg:pt-18">
          <Container>
            <div className="grid gap-10 xl:grid-cols-[0.92fr_1.08fr] xl:items-end">
              <div className="max-w-[40rem]">
                <span className="section-kicker">验证演示</span>
                <h1 className="mt-6 text-[clamp(2.7rem,5vw,4.8rem)] font-semibold leading-[0.98] text-primary-text">
                  一次调试现场，
                  <span className="mt-3 block text-mist-blue">
                    被完整保留成一条可验证、可回放的本地证据链。
                  </span>
                </h1>
                <p className="mt-6 text-[1.04rem] leading-8 text-secondary-text sm:text-[1.12rem]">
                  编辑器帧、终端错误、查阅动作和验证结果进入同一条链路。
                  原始记录在前，回放能力在中间，派生阅读层与候选层始终保持降权。
                </p>

                <div className="mt-8 flex flex-col gap-3 sm:flex-row">
                  <Link href="/demo" className="inline-flex">
                    <Button
                      size="lg"
                      variant="outline"
                      className="h-12 rounded-2xl border-black/8 bg-white/82 px-6 text-base font-medium text-primary-text hover:bg-white"
                    >
                      回到 Demo Hub
                    </Button>
                  </Link>
                  <Button
                    size="lg"
                    onClick={reset}
                    className="h-12 rounded-2xl bg-primary-text px-6 text-base font-semibold text-white hover:bg-primary-text/94"
                  >
                    重播这条证据链
                  </Button>
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4">
                {[
                  {
                    label: "已记录",
                    value: `${observedCount}/6`,
                    detail: "已进入链路的原始记录",
                  },
                  {
                    label: "回放位置",
                    value: formatTime(timelinePosition),
                    detail: "当前回放窗口",
                  },
                  {
                    label: "阶段",
                    value: phaseMeta.label,
                    detail: phaseMeta.title,
                  },
                  {
                    label: "状态",
                    value: isPlaying && !isPaused ? "运行中" : "已暂停",
                    detail: mode.replace(/-/g, " "),
                  },
                ].map((metric) => (
                  <div
                    key={metric.label}
                    className="surface-panel rounded-[1.45rem] p-4"
                  >
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                      {metric.label}
                    </div>
                    <div className="mt-3 text-[1.55rem] font-semibold text-primary-text">{metric.value}</div>
                    <div className="mt-2 text-sm leading-6 text-secondary-text">{metric.detail}</div>
                  </div>
                ))}
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-20 sm:pb-24">
          <Container>
            <div className="grid gap-5 xl:grid-cols-[0.9fr_1.1fr]">
              <ModeSwitcher />

              <div className="surface-panel rounded-[1.55rem] p-4">
                <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
                  <div>
                    <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
                      系统约束
                    </div>
                    <div className="mt-2 text-lg font-semibold text-primary-text">
                      原始证据站在最前面，派生阅读层和候选层保持各自的位置。
                    </div>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    <span className="rounded-full bg-white px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.16em] text-primary-text">
                      本地优先
                    </span>
                    <span className="rounded-full bg-mist-blue/10 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.16em] text-mist-blue">
                      只增不减
                    </span>
                    <span className="rounded-full bg-digital-lavender/10 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.16em] text-digital-lavender">
                      候选从属
                    </span>
                  </div>
                </div>
                <p className="mt-3 text-sm leading-7 text-secondary-text">
                  上下文图负责组织阅读，原始记录负责核对来源，回放功能负责回到发生现场。
                  三层一起工作，但不会互相越级。
                </p>
              </div>
            </div>

            <div className="mt-5 grid gap-5 xl:grid-cols-[1.22fr_0.78fr]">
              <div className="surface-panel-strong rounded-[2rem] p-4 sm:p-5">
                <div className="min-h-[42rem]">
                  <MainDemoView />
                </div>
              </div>

              <div className="min-h-[42rem]">
                <TimelinePanel />
              </div>
            </div>

            <div className="mt-5">
              <ReplayControls />
            </div>

            <div className="mt-5 grid gap-5 xl:grid-cols-[1.08fr_0.92fr]">
              <div className="min-h-[33rem]">
                <InspectorPanel
                  activeEvidenceId={activeEvidenceId}
                  onEvidenceClick={handleEvidenceClick}
                />
              </div>

              <div className="min-h-[33rem]">
                <GraphView activeNodeId={activeNodeId} onNodeClick={handleNodeClick} />
              </div>
            </div>

            <div className="mt-5">
              <ProofStrip onBadgeClick={handleProofBadgeClick} />
            </div>

            <div className="mt-5 grid gap-5 xl:grid-cols-[1fr_1fr]">
              <div className="min-h-[28rem]">
                <ConsumableResults />
              </div>

              <div className="min-h-[28rem]">
                <SourceTraceDrawer
                  derivedId={selectedDerivedId}
                  onClose={() => setSelectedDerivedId(null)}
                />
              </div>
            </div>
          </Container>
        </section>
      </main>

      <Footer />
    </div>
  );
}

export default function DebugBugDemoPage() {
  return (
    <DemoProvider>
      <DebugBugProofContent />
    </DemoProvider>
  );
}
