"use client";

import { MOCK_EVIDENCE } from "@/lib/demo-data";
import { useDemo } from "@/lib/demo-context";
import { MOCK_GRAPH_NODES } from "@/lib/graph-data";

interface SourceTraceDrawerProps {
  derivedId: string | null;
  onClose?: () => void;
}

export function SourceTraceDrawer({ derivedId, onClose }: SourceTraceDrawerProps) {
  const { activeNodeId } = useDemo();
  const selectedId = derivedId ?? activeNodeId ?? "derived-sequence";
  const selectedNode = MOCK_GRAPH_NODES.find((node) => node.id === selectedId) ?? MOCK_GRAPH_NODES[0];
  const references = selectedNode.evidenceRefs
    .map((ref) => MOCK_EVIDENCE.find((record) => record.id === ref))
    .filter((record): record is NonNullable<typeof record> => Boolean(record));

  return (
    <div className="surface-panel h-full rounded-[1.8rem] p-5">
      <div className="flex items-start justify-between gap-4 border-b border-black/6 pb-4">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
            Source trace
          </div>
          <h3 className="mt-2 text-xl font-semibold text-primary-text">当前节点的来源关系</h3>
          <p className="mt-2 text-sm leading-7 text-secondary-text">
            选中 graph 中的任意节点后，这里会把它回指到具体证据，确保派生层和候选层始终有来源。
          </p>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="rounded-full border border-black/6 bg-white/78 px-3 py-1.5 text-[11px] font-medium text-secondary-text transition-colors hover:bg-white"
        >
          清空
        </button>
      </div>

      <div className="mt-5 rounded-[1.4rem] border border-black/6 bg-white/82 p-4">
        <div className="flex flex-wrap items-center gap-2">
          <span className="rounded-full bg-warm-card px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-secondary-text">
            {selectedNode.layer}
          </span>
          <span className="text-sm font-semibold text-primary-text">{selectedNode.label}</span>
        </div>
        <p className="mt-3 text-sm leading-7 text-secondary-text">{selectedNode.detail}</p>
      </div>

      <div className="mt-4 space-y-3">
        {references.map((record) => (
          <div
            key={record.id}
            className="rounded-[1.35rem] border border-black/6 bg-white/82 p-4"
          >
            <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/66">
              {record.id}
            </div>
            <div className="mt-2 text-sm font-semibold text-primary-text">{record.label}</div>
            <p className="mt-2 text-sm leading-7 text-secondary-text">{record.data.note ?? record.data.content}</p>
          </div>
        ))}
      </div>
    </div>
  );
}
