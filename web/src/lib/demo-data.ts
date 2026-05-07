// Mock evidence records for the proof lane demo.
// The raw chain is the source of truth; any higher-level interpretation must stay bounded to these records.

export interface EvidenceRecord {
  id: string;
  timestamp: number; // seconds from start (0-50)
  type: "screen" | "window" | "event" | "terminal";
  label: string;
  data: {
    windowTitle?: string;
    processName?: string;
    content?: string;
    errorText?: string;
    snippet?: string;
    command?: string;
    filePath?: string;
    note?: string;
  };
}

export const MOCK_EVIDENCE: EvidenceRecord[] = [
  {
    id: "ev-001",
    timestamp: 0,
    type: "screen",
    label: "编辑器帧被保留下来",
    data: {
      windowTitle: "VS Code - src/components/EventTimeline.tsx",
      processName: "Code",
      content: "用户在崩溃前一直停留在事件时间线组件，焦点窗口和可见代码片段被记录下来。",
      snippet: "const rows = payload.entries.map((entry) => entry.label)",
      filePath: "src/components/EventTimeline.tsx",
      note: "系统只保存这一刻看见了什么，不替用户解释原因。",
    },
  },
  {
    id: "ev-002",
    timestamp: 8,
    type: "terminal",
    label: "终端把报错原样收住",
    data: {
      windowTitle: "Terminal - pnpm dev",
      processName: "Windows Terminal",
      errorText: "TypeError: Cannot read properties of undefined (reading 'map')",
      command: "pnpm dev",
      content: "渲染期间直接抛出异常，错误文本以原始终端输出保留。",
      note: "这条记录是 raw truth，不需要任何候选层才能成立。",
    },
  },
  {
    id: "ev-003",
    timestamp: 16,
    type: "window",
    label: "文档查阅进入证据链",
    data: {
      windowTitle: "Chrome - React docs / conditional rendering",
      processName: "Chrome",
      content: "随后用户查阅如何在数据未准备好时安全渲染列表，这个外部研究动作也成为时间线的一部分。",
      snippet: "Render a fallback before mapping an unknown collection.",
      note: "研究行为被记录为活动痕迹，而不是被直接当成结论。",
    },
  },
  {
    id: "ev-004",
    timestamp: 26,
    type: "screen",
    label: "修补动作再次落回编辑器",
    data: {
      windowTitle: "VS Code - src/components/EventTimeline.tsx",
      processName: "Code",
      content: "用户回到代码窗口，给列表渲染增加空数组兜底与空态分支。",
      snippet: "const rows = payload?.entries ?? []",
      filePath: "src/components/EventTimeline.tsx",
      note: "这里仍然只是原始片段，不代表系统已经理解完整意图。",
    },
  },
  {
    id: "ev-005",
    timestamp: 36,
    type: "terminal",
    label: "验证命令确认修补有效",
    data: {
      windowTitle: "Terminal - pnpm test EventTimeline",
      processName: "Windows Terminal",
      command: "pnpm test EventTimeline",
      content: "测试通过，崩溃消失；验证动作本身也作为原始事件进入链路。",
      note: "验证结果仍然是 evidence，不是系统替你写出的总结。",
    },
  },
  {
    id: "ev-006",
    timestamp: 46,
    type: "event",
    label: "这条调试链路现在可以被回放",
    data: {
      windowTitle: "Pulseclaw - Replay summary",
      processName: "Pulseclaw",
      content: "6 条原始记录、1 段可回放轨迹、2 条受约束的候选解释已整理完毕。",
      note: "候选分析只是附着在引用关系上的叠加层，不能回写 raw。",
    },
  },
];

export function getEvidenceAtTime(time: number): EvidenceRecord | undefined {
  const candidates = MOCK_EVIDENCE.filter((record) => record.timestamp <= time);
  return candidates.at(-1) ?? MOCK_EVIDENCE[0];
}

export function getEvidenceUpToTime(time: number): EvidenceRecord[] {
  return MOCK_EVIDENCE.filter((record) => record.timestamp <= time);
}
