"use client";

import {
  GraphNode,
  GraphNodeLayer,
  MOCK_GRAPH_EDGES,
  MOCK_GRAPH_NODES,
  getNodesAtTime,
} from "@/lib/graph-data";
import { useDemo } from "@/lib/demo-context";
import { cn } from "@/lib/utils";

interface GraphViewProps {
  activeNodeId?: string | null;
  onNodeClick?: (nodeId: string) => void;
}

const layerLabels: Record<GraphNodeLayer, string> = {
  raw: "raw truth",
  derived: "derived",
  candidate: "candidate only",
  result: "verified",
};

function getNodeClasses(node: GraphNode, isActive: boolean, isSelected: boolean) {
  const base =
    "absolute min-w-[7.2rem] -translate-x-1/2 -translate-y-1/2 rounded-[1.05rem] border px-3 py-2 text-left shadow-[0_14px_26px_rgba(45,42,38,0.06)] transition-all duration-200";

  if (node.layer === "raw") {
    return cn(
      base,
      isSelected
        ? "border-mist-blue/30 bg-white text-primary-text"
        : isActive
          ? "border-mist-blue/18 bg-white/92 text-primary-text"
          : "border-black/6 bg-white/78 text-primary-text"
    );
  }

  if (node.layer === "derived") {
    return cn(
      base,
      "border-mist-blue/18 bg-mist-blue/8 text-primary-text",
      !isSelected && !isActive && "opacity-82"
    );
  }

  if (node.layer === "candidate") {
    return cn(
      base,
      "border border-dashed border-digital-lavender/28 bg-digital-lavender/7 text-primary-text",
      !isSelected && !isActive && "opacity-74"
    );
  }

  return cn(
    base,
    isSelected
      ? "border-sage-green/28 bg-sage-green/12 text-primary-text"
      : "border-sage-green/18 bg-sage-green/10 text-primary-text"
  );
}

function getEdgeStroke(edgeConfidence: GraphNode["confidence"]) {
  if (edgeConfidence === "high") return "rgba(45, 42, 38, 0.22)";
  if (edgeConfidence === "medium") return "rgba(123, 158, 172, 0.36)";
  return "rgba(184, 169, 201, 0.32)";
}

export function GraphView({ activeNodeId, onNodeClick }: GraphViewProps) {
  const { timelinePosition } = useDemo();
  const activeNodes = getNodesAtTime(timelinePosition);
  const selectedNode =
    MOCK_GRAPH_NODES.find((node) => node.id === activeNodeId) ??
    activeNodes.at(-1) ??
    MOCK_GRAPH_NODES[0];

  return (
    <div className="surface-panel h-full rounded-[1.8rem] p-5">
      <div className="flex items-start justify-between gap-4 border-b border-black/6 pb-4">
        <div>
          <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
            Candidate Graph
          </div>
          <h3 className="mt-2 text-xl font-semibold text-primary-text">派生层与候选层的边界</h3>
          <p className="mt-2 text-sm leading-7 text-secondary-text">
            Graph 用来帮助阅读这条链路，但它不替代 raw evidence。派生节点和候选节点都必须回指原始记录。
          </p>
        </div>
        <div className="flex flex-wrap justify-end gap-2">
          {(["raw", "derived", "candidate"] as GraphNodeLayer[]).map((layer) => (
            <span
              key={layer}
              className={cn(
                "rounded-full px-3 py-1.5 text-[10px] font-semibold uppercase tracking-[0.16em]",
                layer === "raw"
                  ? "bg-white text-primary-text"
                  : layer === "derived"
                    ? "bg-mist-blue/10 text-mist-blue"
                    : "bg-digital-lavender/10 text-digital-lavender"
              )}
            >
              {layerLabels[layer]}
            </span>
          ))}
        </div>
      </div>

      <div className="mt-5 rounded-[1.55rem] border border-black/6 bg-[linear-gradient(135deg,rgba(255,255,255,0.95),rgba(123,158,172,0.05),rgba(184,169,201,0.08))] p-4">
        <div className="rounded-[1.3rem] border border-black/6 bg-white/86 p-3">
          <div className="text-[11px] font-semibold uppercase tracking-[0.16em] text-secondary-text/68">
            Raw first
          </div>
          <p className="mt-2 text-sm leading-7 text-secondary-text">
            顶部是原始事实层，中间是派生阅读层，右下是候选解释。层级被故意拉开，避免把推断读成真相。
          </p>
        </div>

        <div className="relative mt-4 min-h-[24rem] rounded-[1.35rem] border border-black/6 bg-white/82 p-4">
          <svg className="pointer-events-none absolute inset-0 h-full w-full" viewBox="0 0 100 100" preserveAspectRatio="none">
            {MOCK_GRAPH_EDGES.map((edge) => {
              const source = MOCK_GRAPH_NODES.find((node) => node.id === edge.source);
              const target = MOCK_GRAPH_NODES.find((node) => node.id === edge.target);

              if (!source?.position || !target?.position) {
                return null;
              }

              return (
                <line
                  key={edge.id}
                  x1={source.position.x}
                  y1={source.position.y}
                  x2={target.position.x}
                  y2={target.position.y}
                  stroke={getEdgeStroke(edge.confidence)}
                  strokeWidth={edge.confidence === "high" ? 0.7 : 0.5}
                  strokeDasharray={edge.confidence === "low" ? "2.5 2.5" : edge.confidence === "medium" ? "3 1.8" : "0"}
                />
              );
            })}
          </svg>

          <div className="absolute left-4 top-4 text-[10px] font-semibold uppercase tracking-[0.18em] text-secondary-text/60">
            Raw truth lane
          </div>
          <div className="absolute left-4 top-[46%] text-[10px] font-semibold uppercase tracking-[0.18em] text-secondary-text/60">
            Derived lane
          </div>
          <div className="absolute left-4 bottom-4 text-[10px] font-semibold uppercase tracking-[0.18em] text-secondary-text/60">
            Candidate + verification lane
          </div>

          {MOCK_GRAPH_NODES.map((node) => {
            if (!node.position) {
              return null;
            }

            const isActive = activeNodes.some((activeNode) => activeNode.id === node.id);
            const isSelected = selectedNode.id === node.id;

            return (
              <button
                key={node.id}
                type="button"
                onClick={() => onNodeClick?.(node.id)}
                className={getNodeClasses(node, isActive, isSelected)}
                style={{ left: `${node.position.x}%`, top: `${node.position.y}%` }}
              >
                <div className="text-[10px] font-semibold uppercase tracking-[0.16em] text-secondary-text/62">
                  {layerLabels[node.layer]}
                </div>
                <div className="mt-1 text-sm font-semibold text-primary-text">{node.label}</div>
              </button>
            );
          })}
        </div>

        <div className="mt-4 rounded-[1.25rem] border border-black/6 bg-white/82 p-4">
          <div className="flex flex-wrap items-center gap-2">
            <span className="rounded-full bg-warm-card px-2.5 py-1 text-[10px] font-semibold uppercase tracking-[0.16em] text-secondary-text">
              selected
            </span>
            <span className="text-sm font-semibold text-primary-text">{selectedNode.label}</span>
          </div>
          <p className="mt-3 text-sm leading-7 text-secondary-text">{selectedNode.detail}</p>
          <div className="mt-3 flex flex-wrap gap-2">
            {selectedNode.evidenceRefs.map((ref) => (
              <span
                key={ref}
                className="rounded-full border border-black/6 bg-white px-2.5 py-1 text-[11px] font-medium text-secondary-text"
              >
                {ref}
              </span>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
