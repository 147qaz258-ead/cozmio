import { CheckCircle2, Download, Lock, Monitor, Shield, Sparkles, Zap } from "lucide-react";
import { siteConfig } from "@/lib/site-config";
import { CozBadge, CozButton, CozCard, CozIconOrb, CozSection } from "./Primitives";
import { CtaBand, DesktopPanel, ExecutorStrip, FlowRail } from "./VisualPanels";
import { CozPageShell } from "./Shell";

export function DesktopPage() {
  return (
    <CozPageShell>
      <CozSection className="grid items-center gap-10 pt-12 lg:grid-cols-[.78fr_1.22fr]">
        <div>
          <CozBadge><Monitor className="h-4 w-4" />DESKTOP NODE</CozBadge>
          <h1 className="mt-8 text-[46px] font-black leading-[1.1] tracking-tight md:text-[58px]">
            把你的电脑，<br />变成 Agent 网络里的<br />本地节点。
          </h1>
          <h2 className="mt-6 text-3xl font-bold">越用，系统越理解你。</h2>
          <p className="mt-7 max-w-[560px] text-lg leading-9 text-[#625b54]">Cozmio Desktop Node 在本地执行任务、沉淀长期记忆、理解真实项目上下文，并安全地连接你的执行端。你的电脑不只是工具，而是 Agent 网络中的信任节点。</p>
          <div className="mt-9 flex flex-wrap gap-5">
            <CozButton href={siteConfig.downloads.windows}>下载 Desktop App<Download className="h-4 w-4" /></CozButton>
            <CozButton href="/request" variant="light">申请内测</CozButton>
          </div>
          <div className="mt-10 flex flex-wrap gap-4">
            {["数据本地优先", "端到端加密", "你完全掌控"].map((item) => <CozBadge key={item}><Shield className="h-4 w-4" />{item}</CozBadge>)}
          </div>
        </div>
        <DesktopPanel />
      </CozSection>

      <CozSection className="mt-12 grid gap-6 md:grid-cols-3">
        {[
          ["本地执行", "在你的电脑上运行任务，访问本地文件、工具与环境，低延迟、更可靠。", Monitor, "gold"],
          ["长期记忆", "沉淀你的项目、偏好与决策脉络，时间越长，Agent 越懂你。", Sparkles, "violet"],
          ["连接执行端", "连接 Claude Code、Codex、Gemini 或自定义 CLI，让 Agent 调度真实的执行能力。", Zap, "orange"],
        ].map(([title, body, Icon, tone]) => (
          <CozCard key={title as string} className="overflow-hidden p-8">
            <CozIconOrb icon={Icon as typeof Monitor} tone={tone as "gold"} />
            <h2 className="mt-7 text-3xl font-bold">{title as string}</h2>
            <p className="mt-4 text-base leading-8 text-[#6d665f]">{body as string}</p>
            <div className="mt-8 h-32 rounded-3xl bg-[radial-gradient(circle_at_50%_55%,rgba(245,181,68,.34),transparent_44%),linear-gradient(135deg,rgba(255,255,255,.7),rgba(244,235,222,.62))]" />
          </CozCard>
        ))}
      </CozSection>

      <CozSection className="mt-10">
        <h2 className="mb-3 text-3xl font-bold">从理解到交付，完整闭环。</h2>
        <p className="mb-7 text-[#736c64]">你的节点在每一步都参与其中。</p>
        <FlowRail mode="desktop" />
      </CozSection>

      <CozSection className="mt-10">
        <div className="coz-card p-8">
          <h2 className="text-3xl font-bold">已连接执行端</h2>
          <p className="mt-2 text-[#736c64]">灵活接入，按需扩展。</p>
          <div className="mt-8"><ExecutorStrip /></div>
        </div>
      </CozSection>

      <CozSection className="mt-10">
        <div className="coz-card p-8">
          <h2 className="text-3xl font-bold">你的数据，你的规则。</h2>
          <p className="mt-2 text-[#736c64]">Cozmio Desktop Node 以隐私优先与可控为核心设计。</p>
          <div className="mt-8 grid gap-5 md:grid-cols-5">
            {[
              ["本地优先", "数据与记忆默认保存在本地。", Monitor],
              ["端到端加密", "与云端通信均执行加密连接。", Lock],
              ["仅限可控", "所有操作可审计。", Shield],
              ["透明可审计", "完整记录节点活动与调用。", CheckCircle2],
              ["开源可验证", "核心组件开源，社区审计。", Sparkles],
            ].map(([title, text, Icon]) => (
              <CozCard key={title as string} className="p-5">
                <Icon className="h-5 w-5 text-[#d9961d]" />
                <h3 className="mt-4 font-bold">{title as string}</h3>
                <p className="mt-3 text-sm leading-6 text-[#756e66]">{text as string}</p>
              </CozCard>
            ))}
          </div>
          <div className="mt-7 flex flex-wrap justify-center gap-3">
            {["SOC 2 Ready", "ISO 27001 设计原则", "本地隐私优先", "最小权限原则"].map((item) => <CozBadge key={item}>{item}</CozBadge>)}
          </div>
        </div>
      </CozSection>

      <CozSection><CtaBand title="立即启用 Desktop Node，让 Cozmio 真正理解并协同你的工作。" body="从你的真实项目、执行工具和长期记忆开始，让 Agent 进入可以交付的工作现场。" primary="下载 Desktop App" secondary="申请内测" primaryHref={siteConfig.downloads.windows} secondaryHref="/request" /></CozSection>
    </CozPageShell>
  );
}
