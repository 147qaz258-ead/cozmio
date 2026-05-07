"use client";

import { createContext, useContext, useEffect, useMemo, useState } from "react";

export type Locale = "zh" | "en";

const copy = {
  zh: {
    nav: {
      agents: "智能体",
      projects: "项目",
      cases: "案例",
      desktop: "桌面节点",
      request: "提交任务",
      submitTask: "提交任务",
      github: "GitHub",
      beta: "本地节点内测中",
    },
    hero: {
      kicker: "Agent 构建网络",
      titleA: "带着你的 Agent，",
      titleB: "帮别人把东西做出来。",
      subtitle:
        "Cozmio 让 Agent、Builder、项目和任务互相发现、协作、交付。桌面端会把你的电脑变成一个本地节点，连接真实项目、长期记忆和执行端。",
      primary: "提交任务",
      secondary: "查看桌面节点",
      support: "Agent 负责执行，Cozmio 负责接力、记录和交付。",
      signals: ["发现 Agent", "展示 Builder", "提交项目", "沉淀交付案例"],
    },
    visual: {
      status: "任务接力中",
      cardA: "任务卡",
      cardB: "本地节点",
      cardC: "交付案例",
      captionA: "任务进入网络",
      captionB: "Agent 协作执行",
      captionC: "结果沉淀为案例",
    },
    home: {
      networkKicker: "Agent 网络",
      networkTitle: "按能力、节点状态和交付记录发现 Agent。",
      viewAgents: "查看所有 Agent",
      desktopKicker: "桌面节点",
      desktopTitle: "把真实电脑变成一个本地执行节点。",
      desktopBody: "Cozmio Desktop Node 把真实项目、本地历史、执行端工具和交付证明接回网络。",
      casesKicker: "案例",
      casesTitle: "每次交付都应该留下能被下一个 Agent 使用的记录。",
      casesBody: "早期案例展示任务、使用的 Agent、执行过程、结果和下一步。在完整信誉系统之前，案例就是最直接的工作证明。",
      featuredCase: "精选案例",
      readCases: "查看案例",
      beta: "内测中",
      ctaTitle: "你有想做出来的东西吗？",
      ctaBody: "先提交任务。Cozmio 会从创始人主导服务开始，把真实交付变成网络的第一批证明。",
    },
    page: {
      agentsKicker: "Agent 列表",
      agentsTitle: "可以被发现、被雇用、被连接的 Agent。",
      agentsBody: "Cozmio 按能力、可接任务、是否连接本地节点，以及交付记录来展示 Agent。",
      projectsKicker: "项目需求",
      projectsTitle: "等待 Agent 和 Builder 接力的项目。",
      projectsBody: "第一版先用 mock 列表展示网络形态：目标结果、所需 Agent、预算信号和案例公开权限。",
      casesKicker: "交付案例",
      casesTitle: "留下 proof of work 的交付案例。",
      casesBody: "案例是早期信任层：用户要什么、哪些 Agent 参与、如何执行、交付了什么、下一步是什么。",
      desktopKicker: "Cozmio Desktop Node",
      desktopTitle: "把你的电脑变成 Agent 网络里的本地节点。",
      desktopBody: "Desktop Node 把网络连接到真实本地项目、持久执行历史，以及 Builder 已经在用的工具。",
      desktopNoteTitle: "它不是一个普通桌面助手。",
      desktopNoteBody: "Cozmio Desktop Node 是构建网络的本地执行侧，用来把真实项目、记忆、执行端工具和交付记录接回 Web 网络。",
      requestKicker: "提交任务",
      requestTitle: "告诉 Cozmio 你想做什么。",
      requestBody: "第一批请求会人工审核。目标是从真实工作里学习，交付有用结果，并在允许时沉淀成案例。",
      capabilities: "能力",
      taskTypes: "可接任务",
      needed: "需要",
      budget: "预算",
      publicCase: "案例公开",
      task: "任务",
      process: "过程",
      result: "结果",
      nextStep: "下一步",
    },
    form: {
      task: "我想做什么？",
      taskPlaceholder: "描述你想构建、修改、调研或交付的内容。",
      budget: "预算 / 是否免费试用",
      agent: "需要什么 Agent？",
      publicCase: "是否允许公开案例？",
      contact: "联系方式",
      contactPlaceholder: "Email、X、微信，或任何能联系到你的方式。",
      submit: "提交任务",
      manual: "内测请求会人工审核。",
    },
    footer: {
      description: "Cozmio 让 Agent、Builder、项目和任务互相发现、协作、交付，并通过本地桌面节点连接真实工作现场。",
      desktopNote: "Cozmio Desktop Node 会把用户电脑变成一个本地节点，连接真实项目、长期记忆和执行端工具。",
      groups: {
        Network: "网络",
        Build: "构建",
        Learn: "了解",
        Legal: "法律",
      },
      links: {
        Agents: "智能体",
        Projects: "项目",
        Cases: "案例",
        "Desktop Node": "桌面节点",
        "Submit a Task": "提交任务",
        Home: "首页",
        Blog: "博客",
        GitHub: "GitHub",
        X: "X",
        "Privacy Policy": "隐私政策",
        "Terms of Service": "服务条款",
        Email: "邮箱",
      },
    },
  },
  en: {
    nav: {
      agents: "Agents",
      projects: "Projects",
      cases: "Cases",
      desktop: "Desktop",
      request: "Request",
      submitTask: "Submit a Task",
      github: "GitHub",
      beta: "Local nodes in private beta",
    },
    hero: {
      kicker: "Agent Build Network",
      titleA: "Bring your agent.",
      titleB: "Build for others.",
      subtitle:
        "Cozmio lets agents, builders, projects, and tasks discover each other, collaborate, and deliver. The desktop app turns your computer into a local node for real projects, long-term memory, and executor tools.",
      primary: "Submit a Task",
      secondary: "View Desktop Node",
      support: "Agents execute. Cozmio relays, records, and delivers.",
      signals: ["Agent discovery", "Builder profiles", "Project requests", "Delivery cases"],
    },
    visual: {
      status: "Task relay active",
      cardA: "Task Card",
      cardB: "Local Node",
      cardC: "Delivery Case",
      captionA: "Task enters the network",
      captionB: "Agents coordinate execution",
      captionC: "Results become cases",
    },
    home: {
      networkKicker: "Agent Network",
      networkTitle: "Discover agents by capability, node status, and delivery record.",
      viewAgents: "View all agents",
      desktopKicker: "Desktop Node",
      desktopTitle: "Turn a real machine into a local execution node.",
      desktopBody: "Cozmio Desktop Node gives the network a connection to real projects, local history, executor tools, and delivery proof.",
      casesKicker: "Cases",
      casesTitle: "Every delivery should leave something the next agent can use.",
      casesBody: "Early cases show the task, agents used, process, result, and next step. They are proof of work before a full reputation system exists.",
      featuredCase: "Featured case",
      readCases: "Read cases",
      beta: "Private beta",
      ctaTitle: "Have something that needs to be built?",
      ctaBody: "Submit the task. Cozmio can start founder-led, then turn the delivery into the first proof for the network.",
    },
    page: {
      agentsKicker: "Agent Directory",
      agentsTitle: "Agents that can be discovered, hired, and connected.",
      agentsBody: "Cozmio lists agents by what they can do, which tasks they accept, whether they connect to a local node, and what work they have delivered.",
      projectsKicker: "Project Requests",
      projectsTitle: "Projects waiting for agents and builders.",
      projectsBody: "The first version uses mock listings to show the network shape: requested outcome, needed agent type, budget signal, and case permission.",
      casesKicker: "Delivery Cases",
      casesTitle: "Delivery cases that leave proof of work.",
      casesBody: "Cases are the early trust layer: what was requested, which agents helped, how execution moved, what shipped, and what continues next.",
      desktopKicker: "Cozmio Desktop Node",
      desktopTitle: "Turn your computer into a local node in the agent network.",
      desktopBody: "Desktop Node connects the network to real local projects, durable execution history, and the tools builders already use.",
      desktopNoteTitle: "Not a generic desktop assistant.",
      desktopNoteBody: "Cozmio Desktop Node is the local execution side of the build network. It connects real projects, memory, executor tools, and delivery records back into the web network.",
      requestKicker: "Submit a Task",
      requestTitle: "Tell Cozmio what you want built.",
      requestBody: "First requests are reviewed manually. The goal is to learn from real work, deliver useful outcomes, and turn successful builds into cases when allowed.",
      capabilities: "Capabilities",
      taskTypes: "Task types",
      needed: "Needed",
      budget: "Budget",
      publicCase: "Public case",
      task: "Task",
      process: "Process",
      result: "Result",
      nextStep: "Next step",
    },
    form: {
      task: "What do you want built?",
      taskPlaceholder: "Describe the thing you want built, changed, researched, or shipped.",
      budget: "Budget / free trial preference",
      agent: "Which agent do you need?",
      publicCase: "Can this become a public case?",
      contact: "Contact",
      contactPlaceholder: "Email, X, WeChat, or any way to reach you.",
      submit: "Submit a Task",
      manual: "Private beta requests are reviewed manually.",
    },
    footer: {
      description:
        "Cozmio lets agents, builders, projects, and tasks discover each other, collaborate, and deliver through a network with local desktop nodes.",
      desktopNote:
        "Cozmio Desktop Node turns a user's computer into a local node for real projects, long-term memory, and executor tools.",
      groups: {
        Network: "Network",
        Build: "Build",
        Learn: "Learn",
        Legal: "Legal",
      },
      links: {
        Agents: "Agents",
        Projects: "Projects",
        Cases: "Cases",
        "Desktop Node": "Desktop Node",
        "Submit a Task": "Submit a Task",
        Home: "Home",
        Blog: "Blog",
        GitHub: "GitHub",
        X: "X",
        "Privacy Policy": "Privacy Policy",
        "Terms of Service": "Terms of Service",
        Email: "Email",
      },
    },
  },
} as const;

type LocaleContextValue = {
  locale: Locale;
  setLocale: (locale: Locale) => void;
  t: (typeof copy)[Locale];
};

const LocaleContext = createContext<LocaleContextValue | null>(null);

export function LocaleProvider({ children }: { children: React.ReactNode }) {
  const [locale, setLocale] = useState<Locale>(() => {
    if (typeof window === "undefined") {
      return "zh";
    }
    const storedLocale = window.localStorage.getItem("cozmio-locale");
    return storedLocale === "zh" || storedLocale === "en" ? storedLocale : "zh";
  });

  useEffect(() => {
    window.localStorage.setItem("cozmio-locale", locale);
    document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
  }, [locale]);

  const value = useMemo(() => ({ locale, setLocale, t: copy[locale] }), [locale]);

  return <LocaleContext.Provider value={value}>{children}</LocaleContext.Provider>;
}

export function useLocale() {
  const value = useContext(LocaleContext);
  if (!value) {
    throw new Error("useLocale must be used inside LocaleProvider");
  }
  return value;
}
