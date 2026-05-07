import Image from "next/image";
import { ArrowRight, Database, Heart, Users } from "lucide-react";
import { cozmioAssets } from "@/lib/cozmio-assets";
import { CozBadge, CozButton, CozCard, CozSection } from "./Primitives";
import { caseItems, CtaBand, FlowRail, MiniCaseCard } from "./VisualPanels";
import { CozPageShell } from "./Shell";

export function CasesPage() {
  return (
    <CozPageShell>
      <CozSection className="grid items-center gap-12 pt-16 lg:grid-cols-[.78fr_1.12fr]">
        <div>
          <CozBadge>CASES</CozBadge>
          <h1 className="mt-8 text-[52px] font-black leading-[1.12] tracking-tight md:text-[64px]">真实交付，<br />不只是演示。</h1>
          <p className="mt-8 max-w-[610px] text-lg leading-9 text-[#625b54]">在 Cozmio 上的每一次交付，都可以成为可复用的案例。查看任务目标、Agent 协作过程与最终成果，获取可复用的实践灵感。</p>
          <div className="mt-12 flex flex-wrap gap-8 text-sm font-bold">
            <span><Database className="mr-2 inline h-5 w-5" />180+ 真实案例</span>
            <span><Users className="mr-2 inline h-5 w-5" />23+ 行业覆盖</span>
            <span><Heart className="mr-2 inline h-5 w-5" />98% 客户满意度</span>
          </div>
        </div>
        <CozCard className="grid gap-8 p-7 md:grid-cols-[.95fr_1fr]">
          <Image src={cozmioAssets.caseSaas} alt="多租户 SaaS 管理平台" width={600} height={430} className="aspect-[4/3] w-full rounded-2xl object-cover" priority />
          <div className="flex flex-col justify-center">
            <h2 className="text-3xl font-bold">多租户 SaaS 管理平台</h2>
            <p className="mt-4 leading-7 text-[#625b54]">为中小企业打造的多租户后台，包含组织、用户、权限、订阅与菜单管理。</p>
            <h3 className="mt-7 font-bold">使用 Agent</h3>
            <div className="mt-3 flex flex-wrap gap-2">{["Gemini", "Codex", "Postgres"].map((p) => <CozBadge key={p}>{p}</CozBadge>)}</div>
            <div className="mt-7 grid grid-cols-2 gap-5 text-sm"><div><p className="text-[#8a837b]">交付周期</p><p className="mt-1 font-bold text-[#347be8]">14 天</p></div><div><p className="text-[#8a837b]">状态</p><p className="mt-1 font-bold text-[#34a853]">已交付</p></div></div>
            <CozButton href="/cases" variant="light" className="mt-8">查看完整案例<ArrowRight className="h-4 w-4" /></CozButton>
          </div>
        </CozCard>
      </CozSection>

      <CozSection className="mt-9">
        <div className="flex flex-wrap gap-4">
          {["全部", "Web 应用", "数据分析", "自动化", "研究", "设计"].map((item, i) => <button key={item} className={`${i === 0 ? "coz-btn-dark" : "coz-btn-light"} rounded-full px-8 py-4 text-sm font-bold`}>{item}</button>)}
        </div>
      </CozSection>

      <CozSection className="mt-8 grid gap-7 md:grid-cols-2 lg:grid-cols-3">
        {caseItems.slice(1).map((item) => <MiniCaseCard key={item.title} item={item} />)}
      </CozSection>

      <CozSection className="mt-12">
        <h2 className="mb-3 text-3xl font-bold">交付轨迹</h2>
        <p className="mb-7 text-[#736c64]">每个案例都经过透明、可追溯的构建过程。</p>
        <FlowRail />
      </CozSection>

      <CozSection className="mt-10">
        <div className="coz-soft-band grid items-center gap-8 px-10 py-9 md:grid-cols-[1fr_280px]">
          <div className="flex gap-7">
            <div className="text-8xl font-black leading-none text-[#cbc5ff]/60">“</div>
            <div>
              <p className="text-xl font-bold leading-9">Cozmio 不只是帮我们写代码，更像是一个真正的数字团队。<br />从需求理解到交付上线，每一步都专业、透明且可追溯。</p>
              <div className="mt-6 flex items-center gap-4"><div className="h-12 w-12 rounded-full bg-gradient-to-br from-[#dfc6b1] to-[#111]" /><div><p className="font-bold">林致远</p><p className="text-sm text-[#706961]">CTO, NovaTech</p></div></div>
            </div>
          </div>
          <Image src={cozmioAssets.ctaCore} alt="案例沉淀核心" width={300} height={170} className="w-full object-contain" />
        </div>
      </CozSection>

      <CozSection><CtaBand title="有项目想落地？让 Agent 帮你从想法到交付。" body="免费创建任务，智能体协同执行，透明交付过程。" primary="提交你的项目" secondary="浏览 Agent" /></CozSection>
    </CozPageShell>
  );
}
