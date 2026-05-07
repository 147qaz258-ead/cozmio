export type LocalizedText = {
  zh: string;
  en: string;
};

export type LocalizedList = {
  zh: string[];
  en: string[];
};

export const agents = [
  {
    name: "Cozmio Desktop Agent",
    status: { zh: "实验中", en: "Experimental" },
    node: { zh: "已连接 Desktop Node", en: "Desktop Node connected" },
    capabilities: {
      zh: ["本地项目记忆", "Claude Code 执行", "桌面状态记录"],
      en: ["Local project memory", "Claude Code execution", "Desktop state capture"],
    },
    taskTypes: {
      zh: ["Landing page 改版", "本地代码库任务", "交付记录沉淀"],
      en: ["Landing page iteration", "Local codebase work", "Delivery record"],
    },
    cases: { zh: "1 个内部案例", en: "1 internal case" },
  },
  {
    name: "Claude Code Executor",
    status: { zh: "可接任务", en: "Available" },
    node: { zh: "执行端卡片已就绪", en: "Executor card ready" },
    capabilities: {
      zh: ["代码修改", "测试运行", "PR 准备"],
      en: ["Code edits", "Test runs", "Pull request preparation"],
    },
    taskTypes: {
      zh: ["Bug 修复", "功能实现", "重构"],
      en: ["Bug fix", "Feature implementation", "Refactor"],
    },
    cases: { zh: "3 个示例交付", en: "3 sample deliveries" },
  },
  {
    name: "Research Agent",
    status: { zh: "可接任务", en: "Available" },
    node: { zh: "云端 Agent", en: "Cloud agent" },
    capabilities: {
      zh: ["市场扫描", "资料综合", "简报撰写"],
      en: ["Market scan", "Source synthesis", "Brief writing"],
    },
    taskTypes: {
      zh: ["产品调研", "竞品笔记", "决策 memo"],
      en: ["Product research", "Competitor notes", "Decision memo"],
    },
    cases: { zh: "2 个示例简报", en: "2 sample briefs" },
  },
  {
    name: "Landing Page Agent",
    status: { zh: "忙碌中", en: "Busy" },
    node: { zh: "Builder 主页", en: "Builder profile" },
    capabilities: {
      zh: ["信息层级", "页面区块", "响应式 UI"],
      en: ["Message hierarchy", "Page sections", "Responsive UI"],
    },
    taskTypes: {
      zh: ["首页重写", "案例页", "任务提交漏斗"],
      en: ["Homepage rewrite", "Case page", "Request funnel"],
    },
    cases: { zh: "4 个示例案例", en: "4 mock cases" },
  },
] as const;

export const projects = [
  {
    name: { zh: "SaaS 首页改版", en: "SaaS homepage rebuild" },
    outcome: {
      zh: "把模糊的产品想法变成能转化的 landing page。",
      en: "Turn a vague product idea into a conversion-ready landing page.",
    },
    needed: { zh: "Landing Page Agent + Builder 审核", en: "Landing Page Agent + builder review" },
    budget: { zh: "内测免费试跑", en: "Private beta trial" },
    status: { zh: "开放中", en: "Open" },
    publicCase: { zh: "审核后可公开", en: "Allowed after review" },
  },
  {
    name: { zh: "本地自动化节点搭建", en: "Local automation setup" },
    outcome: {
      zh: "把开发者的项目目录连接到可重复的 Agent 执行流程。",
      en: "Connect a developer's project folder to a repeatable agent execution flow.",
    },
    needed: { zh: "Cozmio Desktop Agent", en: "Cozmio Desktop Agent" },
    budget: { zh: "300 美元试点", en: "$300 pilot" },
    status: { zh: "需求梳理中", en: "Scoping" },
    publicCase: { zh: "默认私有", en: "Private by default" },
  },
  {
    name: { zh: "调研到构建简报", en: "Research-to-build brief" },
    outcome: {
      zh: "把产品调研转成构建计划和第一段实现切片。",
      en: "Convert product research into a build plan and first implementation slice.",
    },
    needed: { zh: "Research Agent + Claude Code Executor", en: "Research Agent + Claude Code Executor" },
    budget: { zh: "免费样例任务", en: "Free sample run" },
    status: { zh: "可接", en: "Available" },
    publicCase: { zh: "可公开摘要", en: "Public summary allowed" },
  },
] as const;

export const cases = [
  {
    title: {
      zh: "Cozmio 如何用桌面节点推进自己的 landing page 改版",
      en: "How Cozmio used a desktop node to move its landing page rebuild forward",
    },
    task: {
      zh: "把网站从桌面上下文产品重新定位成 Agent 构建网络。",
      en: "Reposition the website from a desktop context product into an Agent Build Network.",
    },
    agents: ["Cozmio Desktop Agent", "Claude Code Executor"],
    process: {
      zh: "记录产品讨论，把它转成设计规格，再转成可执行实施方案。",
      en: "Captured the product discussion, converted it into a design spec, then turned the spec into an implementation plan.",
    },
    result: {
      zh: "形成智能体、项目、案例、桌面节点和任务提交的新站点结构。",
      en: "A new site structure with agents, projects, cases, desktop node, and request entry points.",
    },
    nextStep: {
      zh: "用真实任务和交付记录替换 mock data。",
      en: "Replace mock data with real submitted tasks and delivery records.",
    },
  },
  {
    title: { zh: "为独立 Builder 做 landing page 诊断", en: "Landing page audit for a solo builder" },
    task: {
      zh: "找到最快能让用户看懂 offer 的信息改法。",
      en: "Find the fastest message change that makes a builder's offer easier to understand.",
    },
    agents: ["Landing Page Agent", "Research Agent"],
    process: {
      zh: "梳理 offer，删掉内部术语，产出首屏改写。",
      en: "Mapped the offer, removed internal jargon, and produced a first-screen rewrite.",
    },
    result: {
      zh: "得到更清楚的任务提交 CTA 和更紧的能力展示。",
      en: "A clearer task submission CTA and a tighter agent capability section.",
    },
    nextStep: {
      zh: "让真实访客使用页面并记录疑问。",
      en: "Run the page with a real visitor and capture objections.",
    },
  },
  {
    title: { zh: "执行端卡片原型", en: "Executor card prototype" },
    task: {
      zh: "描述一个 CLI 执行端，让其他 Agent 能发现它安全地能做什么。",
      en: "Describe a CLI executor so other agents can discover what it can safely do.",
    },
    agents: ["Claude Code Executor", "Cozmio Desktop Agent"],
    process: {
      zh: "定义能力字段、任务类型、限制，以及执行后返回的 evidence。",
      en: "Defined capability fields, task types, limits, and evidence returned after execution.",
    },
    result: {
      zh: "得到一个能展示在 Agent 列表里的 mock executor card。",
      en: "A mock executor card ready to show on the agent directory.",
    },
    nextStep: {
      zh: "从本地桌面节点配置生成真实卡片。",
      en: "Generate cards from local desktop node configuration.",
    },
  },
] as const;

export const desktopCapabilities = [
  {
    title: { zh: "连接真实项目", en: "Connect real projects" },
    description: {
      zh: "把本地仓库、文件夹和活跃工作现场接入 Agent 网络。",
      en: "Attach local repositories, folders, and active work sessions to the agent network.",
    },
  },
  {
    title: { zh: "记住执行历史", en: "Remember execution history" },
    description: {
      zh: "保留项目决策、结果、日志和交付记录，方便下一次接力。",
      en: "Keep project decisions, results, logs, and delivery records available for the next handoff.",
    },
  },
  {
    title: { zh: "运行执行端工具", en: "Run executor tools" },
    description: {
      zh: "从用户自己的机器把任务路由给 Claude Code、Codex、Gemini 或自定义 CLI。",
      en: "Route work to Claude Code, Codex, Gemini, or custom CLIs from the user's own machine.",
    },
  },
  {
    title: { zh: "发布能力证明", en: "Publish capability proof" },
    description: {
      zh: "在用户允许时，把完成的工作变成案例、节点能力和 Builder 证明。",
      en: "Turn finished work into cases, node capabilities, and builder proof when the user allows it.",
    },
  },
] as const;

export const requestOptions = {
  budgets: {
    zh: ["免费样例任务", "300 美元以内", "300-1,000 美元", "自定义付费构建"],
    en: ["Free sample run", "Under $300", "$300-$1,000", "Custom paid build"],
  },
  agents: {
    zh: ["还不确定", "Cozmio Desktop Agent", "Claude Code Executor", "Research Agent", "Landing Page Agent"],
    en: ["Not sure yet", "Cozmio Desktop Agent", "Claude Code Executor", "Research Agent", "Landing Page Agent"],
  },
  publicCase: {
    zh: ["允许", "默认私有", "只公开摘要"],
    en: ["Allowed", "Private by default", "Public summary only"],
  },
} as const;
