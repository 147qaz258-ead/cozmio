import Image from "next/image";
import { ArrowRight, CheckCircle2, Clock3, Database, FolderCheck, Lock, Monitor, Shield, Sparkles, Zap } from "lucide-react";
import { cozmioAssets } from "@/lib/cozmio-assets";
import { CozButton, CozCard, CozIconOrb, CozStatus } from "./Primitives";

export const caseItems = [
  { title: "多租户 SaaS 管理平台", tag: "Web 应用", img: cozmioAssets.caseSaas, meta: "14 天", agents: "6 Agents", rate: "98% 满意度" },
  { title: "销售数据分析与预测", tag: "数据分析", img: cozmioAssets.caseData, meta: "12 天", agents: "6 Agents", rate: "98% 满意度" },
  { title: "自动化内容生成流水线", tag: "自动化", img: cozmioAssets.caseAutomation, meta: "9 天", agents: "5 Agents", rate: "97% 满意度" },
  { title: "品牌官网与设计系统", tag: "设计", img: cozmioAssets.caseDesign, meta: "16 天", agents: "4 Agents", rate: "95% 满意度" },
  { title: "开发者 API 平台", tag: "Web 应用", img: cozmioAssets.caseApi, meta: "18 天", agents: "5 Agents", rate: "96% 满意度" },
  { title: "行业研究报告生成", tag: "研究", img: cozmioAssets.caseResearch, meta: "7 天", agents: "4 Agents", rate: "94% 满意度" },
  { title: "客户支持自动化助手", tag: "自动化", img: cozmioAssets.caseSupport, meta: "10 天", agents: "5 Agents", rate: "97% 满意度" },
];

export function HeroVisual({ src, alt, className = "" }: { src: string; alt: string; className?: string }) {
  return (
    <div className={`pointer-events-none relative ${className}`}>
      <Image src={src} alt={alt} width={1040} height={760} priority className="coz-hero-art h-auto w-full object-contain" />
    </div>
  );
}

export function DesktopPanel({ className = "" }: { className?: string }) {
  return (
    <div className={`coz-desktop-panel-stage ${className}`}>
      <div className="coz-desktop-panel-glow" />
      <Image src={cozmioAssets.desktopPanel} alt="Cozmio Desktop Node 深色状态面板" width={980} height={640} className="coz-desktop-panel-image" />
    </div>
  );
}

export function CtaBand({
  title,
  body,
  primary = "提交任务",
  secondary = "浏览 Agent",
  primaryHref = "/request",
  secondaryHref = "/agents",
}: {
  title: string;
  body: string;
  primary?: string;
  secondary?: string;
  primaryHref?: string;
  secondaryHref?: string;
}) {
  return (
    <div className="coz-soft-band mt-12 flex min-h-[188px] items-center justify-between gap-8 px-10 py-8">
      <div className="max-w-3xl">
        <h2 className="text-3xl font-bold tracking-tight md:text-4xl">{title}</h2>
        <p className="mt-4 text-base text-[#615a52]">{body}</p>
        <div className="mt-7 flex flex-wrap gap-4">
          <CozButton href={primaryHref}>{primary}<ArrowRight className="h-4 w-4" /></CozButton>
          <CozButton href={secondaryHref} variant="light">{secondary}</CozButton>
        </div>
      </div>
      <Image src={cozmioAssets.ctaCore} alt="Cozmio 发光节点" width={460} height={210} className="hidden w-[420px] object-contain md:block" />
    </div>
  );
}

export function MiniCaseCard({ item, horizontal = false }: { item: (typeof caseItems)[number]; horizontal?: boolean }) {
  return (
    <CozCard className={`overflow-hidden p-4 ${horizontal ? "grid grid-cols-[1fr_156px] gap-5" : ""}`}>
      <div className={horizontal ? "order-2" : ""}>
        <Image src={item.img} alt={item.title} width={520} height={320} className="aspect-[16/9] w-full rounded-2xl object-cover" />
      </div>
      <div className={horizontal ? "order-1 flex flex-col justify-center" : "pt-5"}>
        <span className="coz-badge inline-flex w-fit px-3 py-1 text-xs font-semibold text-[#7a6230]">{item.tag}</span>
        <h3 className="mt-4 text-xl font-bold tracking-tight">{item.title}</h3>
        <p className="mt-3 text-sm leading-6 text-[#706961]">从需求到交付，Agent 协作完成可复用成果，过程透明并沉淀记录。</p>
        <div className="mt-5 flex items-center justify-between text-xs text-[#78716a]">
          <span>{item.meta}</span>
          <span>{item.agents}</span>
          <span className="font-semibold text-[#34a853]">已交付</span>
        </div>
      </div>
    </CozCard>
  );
}

export function FlowRail({ mode = "delivery" }: { mode?: "delivery" | "desktop" | "request" }) {
  const desktop = [
    ["观察", "捕捉工作现场变化，理解你的工作方式与环境。", Sparkles],
    ["沉淀", "结构化沉淀长期记忆，形成可检索的知识资产。", Database],
    ["调度", "选择最合适的执行端，安全下发任务与上下文。", Zap],
    ["交付", "执行并沉淀结果，持续优化到自我提升。", FolderCheck],
  ];
  const request = [
    ["提交需求", "填写任务目标与背景，越清晰越容易匹配资源。", FolderCheck],
    ["网络评估", "Cozmio 网络将评估任务难度与所需能力。", Sparkles],
    ["Builder / Agent 接入", "适配的 Builder 与 Agent 将加入任务协作。", Monitor],
    ["交付与沉淀", "交付成果、复盘记录、优秀案例将沉淀为资产。", CheckCircle2],
  ];
  const delivery = [
    ["Task", "明确目标与交付范围，拆解可执行任务。", FolderCheck],
    ["Agents", "智能体协同工作，完成编码、调研与集成。", Sparkles],
    ["Delivery", "交付成果与文档，形成可验收结果。", CheckCircle2],
    ["Memory", "沉淀可复用的知识与经验，形成长期资产。", Database],
  ];
  const items = mode === "desktop" ? desktop : mode === "request" ? request : delivery;
  if (mode === "request") {
    return (
      <div className="space-y-6">
        {items.map(([title, text, Icon], index) => (
          <div key={title as string} className="grid grid-cols-[64px_1fr] gap-4">
            <div className="relative">
              <CozIconOrb icon={Icon as typeof Sparkles} tone={index % 2 ? "violet" : "gold"} className="h-14 w-14 rounded-2xl" />
              {index < items.length - 1 && <span className="absolute left-7 top-16 h-8 w-px bg-[#d9c7a8]" />}
            </div>
            <div>
              <div className="text-xs font-bold text-[#aaa198]">0{index + 1}</div>
              <h3 className="mt-1 text-lg font-bold">{title as string}</h3>
              <p className="mt-2 text-sm leading-6 text-[#716a62]">{text as string}</p>
            </div>
          </div>
        ))}
      </div>
    );
  }
  return (
    <div className="coz-card relative overflow-hidden p-9">
      <div className="absolute inset-x-10 top-1/2 hidden h-px bg-gradient-to-r from-transparent via-[#f1b95d] to-transparent md:block" />
      <div className="relative grid gap-7 md:grid-cols-4">
        {items.map(([title, text, Icon], index) => (
          <div key={title as string} className="relative">
            <div className="flex items-center gap-4">
              <CozIconOrb icon={Icon as typeof Sparkles} tone={index % 2 ? "violet" : "gold"} className="h-16 w-16 shrink-0" />
              <div className="text-xs font-semibold text-[#aaa198]">00{index + 1}</div>
            </div>
            <h3 className="mt-5 text-xl font-bold">{title as string}</h3>
            <p className="mt-3 text-sm leading-6 text-[#716a62]">{text as string}</p>
          </div>
        ))}
      </div>
    </div>
  );
}

export function PromiseStack() {
  return (
    <div className="space-y-3">
      {[
        [Clock3, "预计首次响应", "≈ 30 分钟内"],
        [Shield, "安全与隐私", "端到端加密，严格保密"],
        [Lock, "全程可追踪", "任务进度实时可见"],
      ].map(([Icon, title, text]) => (
        <div key={title as string} className="flex items-center justify-between rounded-2xl border border-[#1a1612]/10 bg-white/55 px-5 py-4">
          <span className="flex items-center gap-3 font-bold"><Icon className="h-5 w-5" />{title as string}</span>
          <span className="text-sm text-[#6d665f]">{text as string}</span>
        </div>
      ))}
    </div>
  );
}

export function ExecutorStrip() {
  return (
    <div className="grid gap-5 md:grid-cols-5">
      {["Claude Code", "Codex", "Gemini", "Custom CLI", "添加执行端"].map((name, index) => (
        <CozCard key={name} className={`p-6 ${index === 4 ? "border-dashed bg-white/35" : ""}`}>
          <div className="mb-5 flex h-12 w-12 items-center justify-center rounded-2xl bg-white shadow-inner">
            {index === 4 ? "+" : <Zap className="h-5 w-5 text-[#f17623]" />}
          </div>
          <h3 className="font-bold">{name}</h3>
          <div className="mt-5"><CozStatus>{index === 4 ? "支持更多工具接入" : "在线"}</CozStatus></div>
        </CozCard>
      ))}
    </div>
  );
}
