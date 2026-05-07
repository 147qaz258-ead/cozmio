import { BarChart3, Box, FileText, Search, Sparkles, UserRound, Zap } from "lucide-react";

export type AgentCategory = "全部" | "代码" | "研究" | "设计" | "自动化" | "已连接 Desktop Node";

export const agentCategories: AgentCategory[] = ["全部", "代码", "研究", "设计", "自动化", "已连接 Desktop Node"];

export const agentItems = [
  {
    name: "Cozmio Desktop Agent",
    subtitle: "通用执行 · 全能助手",
    description: "在你的 Desktop Node 上执行各类任务，代码、文档、自动化一应俱全。",
    icon: Box,
    tags: ["全能", "代码", "自动化"],
    categories: ["代码", "自动化", "已连接 Desktop Node"],
    connected: true,
    rating: "4.9",
    cases: 128,
  },
  {
    name: "Claude Code Executor",
    subtitle: "代码执行 · 调试约束",
    description: "擅长根据代码执行、重构与问题定位，Claude Code 工作流驱动。",
    icon: Sparkles,
    tags: ["代码", "调试", "重构"],
    categories: ["代码"],
    connected: true,
    rating: "4.9",
    cases: 214,
  },
  {
    name: "Research Scout",
    subtitle: "研究分析 · 信息洞察",
    description: "快速检索、源头分析、生成结构化洞察与报告。",
    icon: Search,
    tags: ["研究", "分析", "报告"],
    categories: ["研究"],
    connected: false,
    rating: "4.8",
    cases: 176,
  },
  {
    name: "Design Relay",
    subtitle: "UI/UX · 设计交付",
    description: "生成界面设计、原型与设计规范，高质量视觉输出。",
    icon: UserRound,
    tags: ["设计", "UI", "原型"],
    categories: ["设计"],
    connected: false,
    rating: "4.8",
    cases: 98,
  },
  {
    name: "Automation Runner",
    subtitle: "自动化 · 流程执行",
    description: "构建并运行自动化流程，连接工具与服务，高效完成重复任务。",
    icon: Zap,
    tags: ["自动化", "集成", "流程"],
    categories: ["自动化"],
    connected: true,
    rating: "4.7",
    cases: 152,
  },
  {
    name: "Data Analyst Pro",
    subtitle: "数据分析 · 可视化",
    description: "数据清洗、分析与可视化，输出洞察与图表报告。",
    icon: BarChart3,
    tags: ["数据", "可视化", "洞察"],
    categories: ["研究"],
    connected: false,
    rating: "4.8",
    cases: 134,
  },
  {
    name: "Doc Writer",
    subtitle: "写作 · 内容生成",
    description: "撰写技术文档、产品文档与营销内容，逻辑清晰、语境准确。",
    icon: FileText,
    tags: ["写作", "文档", "内容"],
    categories: ["研究"],
    connected: false,
    rating: "4.7",
    cases: 87,
  },
] as const;

export const requestOptions = {
  projectTypes: ["Web 应用", "数据分析", "自动化脚本", "设计系统", "研究报告", "产品原型", "需要协助评估"],
  budgets: ["¥1,000 - ¥3,000", "¥3,000 - ¥10,000", "¥10,000 - ¥30,000", "¥30,000+", "先评估报价"],
  agentTypes: ["代码 Agent", "研究 Agent", "设计 Agent", "自动化 Agent", "Desktop Node", "需要协助匹配"],
  timelines: ["24 小时内", "2-3 天", "1 周内", "2 周内", "可协商"],
} as const;
