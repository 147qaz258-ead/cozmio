// 50-second autoplay script for the debug proof lane.
// The phases stay grounded in evidence handling: observe, preserve, replay.

export interface ScriptStep {
  id: string;
  timestamp: number;
  phase: "capture" | "record" | "replay";
  label: string;
  description: string;
  evidenceId: string;
  isSignificant: boolean;
  action?: {
    type: "highlight_node" | "show_evidence" | "update_timeline";
    target: string;
  };
}

export const DEMO_SCRIPT: ScriptStep[] = [
  {
    id: "step-01",
    timestamp: 0,
    phase: "capture",
    label: "编辑器焦点被保留",
    description: "代码窗口和可见片段先进入原始证据层。",
    evidenceId: "ev-001",
    isSignificant: true,
    action: {
      type: "highlight_node",
      target: "raw-editor",
    },
  },
  {
    id: "step-02",
    timestamp: 8,
    phase: "capture",
    label: "错误文本被原样记录",
    description: "终端异常原封不动进入 append-only 轨迹。",
    evidenceId: "ev-002",
    isSignificant: true,
    action: {
      type: "highlight_node",
      target: "raw-error",
    },
  },
  {
    id: "step-03",
    timestamp: 16,
    phase: "record",
    label: "查阅动作接入证据链",
    description: "外部研究成为后续回放里可核对的一段上下文。",
    evidenceId: "ev-003",
    isSignificant: false,
    action: {
      type: "highlight_node",
      target: "raw-docs",
    },
  },
  {
    id: "step-04",
    timestamp: 26,
    phase: "record",
    label: "修补动作再次落回原始帧",
    description: "回到编辑器修改代码，候选分析开始有了引用基础。",
    evidenceId: "ev-004",
    isSignificant: true,
    action: {
      type: "highlight_node",
      target: "derived-sequence",
    },
  },
  {
    id: "step-05",
    timestamp: 36,
    phase: "replay",
    label: "验证命令证明修补有效",
    description: "测试通过，但验证结果仍然只是 raw evidence 的一部分。",
    evidenceId: "ev-005",
    isSignificant: true,
    action: {
      type: "highlight_node",
      target: "result-verified",
    },
  },
  {
    id: "step-06",
    timestamp: 46,
    phase: "replay",
    label: "证据链整理完毕，可随时回放",
    description: "此时可以挂载候选解释，但它们仍不得覆盖原始记录。",
    evidenceId: "ev-006",
    isSignificant: false,
    action: {
      type: "highlight_node",
      target: "candidate-guard",
    },
  },
];

export const PHASE_CONFIG = {
  capture: {
    label: "接住",
    title: "先观察",
    description: "系统只保存刚发生的片段与错误，不预先下结论。",
    timeRange: [0, 15] as [number, number],
  },
  record: {
    label: "保留",
    title: "再保留",
    description: "查阅、修改、验证都作为连续证据接入同一条轨迹。",
    timeRange: [15, 35] as [number, number],
  },
  replay: {
    label: "回放",
    title: "最后回放",
    description: "原始记录可以回放，候选解释只能附着其上。",
    timeRange: [35, 50] as [number, number],
  },
};

export function getCurrentPhase(time: number): "capture" | "record" | "replay" {
  if (time < 15) return "capture";
  if (time < 35) return "record";
  return "replay";
}

export function getStepAtTime(time: number): ScriptStep | undefined {
  return DEMO_SCRIPT.find((step, index) => {
    const nextStep = DEMO_SCRIPT[index + 1];
    const endTime = nextStep ? nextStep.timestamp : 50;
    return time >= step.timestamp && time < endTime;
  });
}

export function getStepsUpToTime(time: number): ScriptStep[] {
  return DEMO_SCRIPT.filter((step) => step.timestamp <= time);
}

export function getSignificantSteps(): ScriptStep[] {
  return DEMO_SCRIPT.filter((step) => step.isSignificant);
}

export function getNextSignificantStep(currentTime: number): ScriptStep | undefined {
  return DEMO_SCRIPT.find((step) => step.isSignificant && step.timestamp > currentTime);
}
