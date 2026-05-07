import { MOCK_EVIDENCE } from "@/lib/demo-data";

export type GraphNodeLayer = "raw" | "derived" | "candidate" | "result";

export interface GraphNode {
  id: string;
  layer: GraphNodeLayer;
  label: string;
  detail: string;
  confidence: "high" | "medium" | "low";
  evidenceRefs: string[];
  position?: { x: number; y: number };
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  relationship: "supports" | "follows" | "grounds";
  confidence: "high" | "medium" | "low";
}

export const MOCK_GRAPH_NODES: GraphNode[] = [
  {
    id: "raw-editor",
    layer: "raw",
    label: "编辑器帧",
    detail: "代码窗口仍在焦点内，问题发生前的可见片段被保留下来。",
    confidence: "high",
    evidenceRefs: ["ev-001"],
    position: { x: 14, y: 18 },
  },
  {
    id: "raw-error",
    layer: "raw",
    label: "终端报错",
    detail: "TypeError 文本原样保存，是最直接的原始信号之一。",
    confidence: "high",
    evidenceRefs: ["ev-002"],
    position: { x: 38, y: 18 },
  },
  {
    id: "raw-docs",
    layer: "raw",
    label: "文档查阅",
    detail: "浏览器里查阅 React 条件渲染，这是一段可回放的研究行为。",
    confidence: "high",
    evidenceRefs: ["ev-003"],
    position: { x: 64, y: 18 },
  },
  {
    id: "raw-patch",
    layer: "raw",
    label: "代码修补",
    detail: "空数组兜底与空态分支被实际写回编辑器视图。",
    confidence: "high",
    evidenceRefs: ["ev-004"],
    position: { x: 88, y: 18 },
  },
  {
    id: "derived-sequence",
    layer: "derived",
    label: "调试序列",
    detail: "系统可以把 error → docs → patch 读成一条连续过程，但这仍是派生层。",
    confidence: "medium",
    evidenceRefs: ["ev-002", "ev-003", "ev-004"],
    position: { x: 34, y: 56 },
  },
  {
    id: "candidate-guard",
    layer: "candidate",
    label: "候选解释",
    detail: "较可能的问题是 payload.entries 缺少保护；这是候选，不是事实层。",
    confidence: "low",
    evidenceRefs: ["ev-002", "ev-004"],
    position: { x: 68, y: 56 },
  },
  {
    id: "result-verified",
    layer: "result",
    label: "验证通过",
    detail: "测试成功与回放摘要一起表明这条链路已经闭合。",
    confidence: "high",
    evidenceRefs: ["ev-005", "ev-006"],
    position: { x: 76, y: 84 },
  },
];

export const MOCK_GRAPH_EDGES: GraphEdge[] = [
  {
    id: "edge-1",
    source: "raw-editor",
    target: "raw-error",
    relationship: "follows",
    confidence: "high",
  },
  {
    id: "edge-2",
    source: "raw-error",
    target: "raw-docs",
    relationship: "follows",
    confidence: "high",
  },
  {
    id: "edge-3",
    source: "raw-docs",
    target: "raw-patch",
    relationship: "follows",
    confidence: "high",
  },
  {
    id: "edge-4",
    source: "raw-error",
    target: "derived-sequence",
    relationship: "grounds",
    confidence: "medium",
  },
  {
    id: "edge-5",
    source: "raw-patch",
    target: "candidate-guard",
    relationship: "supports",
    confidence: "low",
  },
  {
    id: "edge-6",
    source: "derived-sequence",
    target: "result-verified",
    relationship: "supports",
    confidence: "medium",
  },
  {
    id: "edge-7",
    source: "candidate-guard",
    target: "result-verified",
    relationship: "supports",
    confidence: "low",
  },
];

export function getNodesAtTime(time: number): GraphNode[] {
  return MOCK_GRAPH_NODES.filter((node) =>
    node.evidenceRefs.some((ref) => {
      const evidence = MOCK_EVIDENCE.find((record) => record.id === ref);
      if (!evidence) return false;
      return Math.abs(evidence.timestamp - time) < 12 || evidence.timestamp <= time;
    })
  );
}
