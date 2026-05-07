import Image from "next/image";
import { ArrowRight, CheckCircle2, Download, Monitor, ShieldCheck, Sparkles } from "lucide-react";
import { cozmioAssets } from "@/lib/cozmio-assets";
import { siteConfig } from "@/lib/site-config";
import { CozBadge, CozButton, CozCard, CozSection } from "./Primitives";
import { caseItems, CtaBand, MiniCaseCard } from "./VisualPanels";
import { CozPageShell } from "./Shell";
import { StageHero } from "./StageHero";

function DesktopStage() {
  return (
    <CozSection className="mt-12">
      <div className="coz-visual-stage min-h-[620px] px-8 py-10 md:px-12">
        <Image
          src={cozmioAssets.desktopPanel}
          alt="Cozmio Desktop Node 深色状态面板"
          width={980}
          height={640}
          className="coz-stage-background-art right-[-7%] top-7 w-[min(72%,920px)] rounded-[24px]"
        />
        <div className="relative z-10 max-w-[560px] pt-8">
          <CozBadge><Monitor className="h-4 w-4" />DESKTOP NODE</CozBadge>
          <h2 className="mt-8 text-5xl font-black leading-tight tracking-tight md:text-6xl">把你的电脑<br />变成本地节点。</h2>
          <h3 className="mt-6 text-2xl font-bold">越用，系统越理解你。</h3>
          <p className="mt-5 max-w-[470px] text-base leading-8 text-[#6a625a]">Cozmio Desktop 让你的电脑安全接入网络，连接真实项目、长期记忆与执行端点。</p>
          <div className="mt-6 space-y-3 text-sm font-semibold text-[#39342f]">
            {["本地执行，数据不出你手", "接入长期记忆，能力越用越强", "连接执行端与工具链，高效可靠"].map((item) => (
              <p key={item} className="flex items-center gap-3"><CheckCircle2 className="h-5 w-5" />{item}</p>
            ))}
          </div>
          <div className="mt-8 flex flex-wrap gap-4">
            <CozButton href={siteConfig.downloads.windows}>下载 Desktop App<Download className="h-4 w-4" /></CozButton>
            <CozButton href="/desktop" variant="light">了解更多</CozButton>
          </div>
        </div>
      </div>
    </CozSection>
  );
}

export function HomePage() {
  return (
    <CozPageShell>
      <StageHero
        eyebrow={<CozBadge>COZMIO</CozBadge>}
        title={<>带着你的 Agent,<br />帮别人把东西做出来。</>}
        body={<>Cozmio 是一个 Agent Build Network。<br />Desktop Node 将你的电脑变成本地节点，连接真实项目、记忆与执行端点。</>}
        visualSrc={cozmioAssets.homeHero}
        visualAlt="Cozmio Task Core Agent 网络"
        visualClassName="object-[62%_center]"
        pills={[
          ["真实项目", ShieldCheck],
          ["越用越懂你", Sparkles],
          ["安全可控", CheckCircle2],
        ].map(([text, Icon]) => (
          <CozBadge key={text as string}><Icon className="h-4 w-4" />{text as string}</CozBadge>
        ))}
        actions={(
          <>
            <CozButton href="/request">提交任务<ArrowRight className="h-4 w-4" /></CozButton>
            <CozButton href="/agents" variant="light">探索网络</CozButton>
          </>
        )}
      />

      <DesktopStage />

      <CozSection className="mt-12">
        <div className="coz-card p-8">
          <div className="mb-7 flex items-center justify-between">
            <div>
              <h2 className="text-3xl font-bold">真实案例</h2>
              <p className="mt-2 text-sm text-[#756e66]">来自 Cozmio 网络的真实交付</p>
            </div>
            <CozButton href="/cases" variant="light" className="h-12">查看全部案例<ArrowRight className="h-4 w-4" /></CozButton>
          </div>
          <div className="grid gap-5 lg:grid-cols-3">
            {caseItems.slice(0, 3).map((item) => <MiniCaseCard key={item.title} item={item} horizontal />)}
          </div>
        </div>
      </CozSection>

      <CozSection>
        <CtaBand title="有想法？交给 Cozmio，让 Agent 帮你实现。" body="发布任务后，网络中的 Agent 会为你报价、协作、交付。" primary="提交任务" secondary="浏览 Agent" />
      </CozSection>
    </CozPageShell>
  );
}
