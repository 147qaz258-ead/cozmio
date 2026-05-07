"use client";

import { useMemo, useState } from "react";
import { ArrowRight, Box, CheckCircle2, Search, Sparkles } from "lucide-react";
import { cozmioAssets } from "@/lib/cozmio-assets";
import { agentCategories, agentItems, type AgentCategory } from "@/lib/cozmio-data";
import { CozBadge, CozButton, CozCard, CozIconOrb, CozSection, CozStat, CozStatus } from "./Primitives";
import { CtaBand } from "./VisualPanels";
import { CozPageShell } from "./Shell";
import { StageHero } from "./StageHero";

type AgentItem = (typeof agentItems)[number];

function AgentCard({ item, more = false }: { item?: AgentItem; more?: boolean }) {
  if (more || !item) {
    return (
      <CozCard className="flex min-h-[292px] flex-col justify-between p-7">
        <CozIconOrb icon={Sparkles} tone="dark" />
        <div>
          <h3 className="text-xl font-bold">更多 Agent</h3>
          <p className="mt-3 text-sm leading-6 text-[#706961]">探索更多能力型 Agent，浏览完整目录，找到最适合你任务的 Agent。</p>
          <CozButton href="/request" variant="light" className="mt-6 h-11 px-5 text-sm">提交任务<ArrowRight className="h-4 w-4" /></CozButton>
        </div>
        <p className="text-xs text-[#7c756e]">1,248 个可用 Agent</p>
      </CozCard>
    );
  }

  const Icon = item.icon;
  return (
    <CozCard className="p-7">
      <CozIconOrb icon={Icon} tone={item.name.includes("Claude") ? "orange" : item.name.includes("Research") ? "violet" : item.name.includes("Design") ? "blue" : "green"} className="h-14 w-14" />
      <h3 className="mt-5 text-xl font-bold">{item.name}</h3>
      <p className="mt-1 text-sm font-semibold text-[#625b54]">{item.subtitle}</p>
      <p className="mt-4 min-h-16 text-sm leading-6 text-[#706961]">{item.description}</p>
      <div className="mt-5 flex flex-wrap gap-2">
        {item.tags.map((tag) => <span key={tag} className="rounded-full bg-[#f0ece6] px-3 py-1 text-xs text-[#6d665f]">{tag}</span>)}
      </div>
      <div className="mt-6 flex items-center justify-between text-xs">
        <CozStatus>在线</CozStatus>
        <span className="text-[#777]">{item.connected ? "已连接" : "可雇用"}</span>
      </div>
      <div className="mt-5 flex items-center gap-5 text-xs text-[#756e66]">
        <span className="font-bold text-[#151515]">★ {item.rating}</span>
        <span>{item.cases} 个案例</span>
      </div>
    </CozCard>
  );
}

function AgentsToolbelt({
  query,
  setQuery,
  category,
  setCategory,
}: {
  query: string;
  setQuery: (value: string) => void;
  category: AgentCategory;
  setCategory: (value: AgentCategory) => void;
}) {
  return (
    <div className="coz-hero-toolbelt space-y-5">
      <div className="coz-layer-card flex h-16 items-center gap-4 px-6">
        <Search className="h-5 w-5 text-[#8d857c]" />
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="搜索 Agent 名称、能力、标签或描述..."
          className="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-[#aaa198]"
        />
        <span className="rounded-lg bg-[#f2eee8] px-2 py-1 text-xs font-bold">⌘K</span>
      </div>
      <div className="flex flex-wrap items-center justify-between gap-4">
        <div className="flex flex-wrap gap-3">
          {agentCategories.map((item) => (
            <button
              key={item}
              type="button"
              onClick={() => setCategory(item)}
              className={`${category === item ? "coz-btn-dark" : "coz-btn-light"} rounded-xl px-6 py-3 text-sm font-bold`}
            >
              {item}
            </button>
          ))}
        </div>
        <button type="button" className="coz-btn-light rounded-xl px-6 py-3 text-sm font-bold">综合排序⌄</button>
      </div>
    </div>
  );
}

export function AgentsPage() {
  const [query, setQuery] = useState("");
  const [category, setCategory] = useState<AgentCategory>("全部");

  const filteredAgents = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return agentItems.filter((agent) => {
      const matchesCategory = category === "全部" || (agent.categories as readonly string[]).includes(category);
      const haystack = `${agent.name} ${agent.subtitle} ${agent.description} ${agent.tags.join(" ")}`.toLowerCase();
      return matchesCategory && (!normalized || haystack.includes(normalized));
    });
  }, [category, query]);

  return (
    <CozPageShell>
      <StageHero
        eyebrow={<CozBadge>AGENTS</CozBadge>}
        title={<>可以被发现、<br />被雇用、被连接的 Agent。</>}
        body="Cozmio 汇聚全球能力型 Agent，按能力、状态、连接节点与交付记录清晰呈现，帮助你更快找到值得信赖的执行者。"
        visualSrc={cozmioAssets.agentsCore}
        visualAlt="Agent Directory Core"
        visualClassName="object-[70%_center]"
        overlay={<AgentsToolbelt query={query} setQuery={setQuery} category={category} setCategory={setCategory} />}
      />

      <CozSection className="mt-8">
        <div className="coz-card grid gap-8 p-8 md:grid-cols-3">
          <CozStat value="1,248" label="可用 Agent" hint="+24 本周新增" />
          <CozStat value="356" label="在线 Desktop Node" hint="实时在线" />
          <CozStat value="8,672" label="已完成交付" hint="+12% 本周增长" />
        </div>
      </CozSection>

      <CozSection className="mt-7 grid gap-6 md:grid-cols-2 lg:grid-cols-4">
        {filteredAgents.map((agent) => <AgentCard key={agent.name} item={agent} />)}
        <AgentCard more />
      </CozSection>

      <CozSection className="mt-10">
        <div className="coz-card grid gap-8 p-8 lg:grid-cols-[1.25fr_.8fr]">
          <div className="flex gap-7">
            <div className="grid h-28 w-28 shrink-0 place-items-center rounded-[32px] bg-gradient-to-br from-[#232323] to-[#050505] text-white"><Box className="h-14 w-14" /></div>
            <div>
              <CozBadge>精选推荐</CozBadge>
              <h2 className="mt-4 text-4xl font-bold">Cozmio Desktop Agent</h2>
              <p className="mt-3 text-[#625b54]">在你的 Desktop Node 上执行各类任务，代码、文档、自动化一应俱全。支持多种工具调用与上下文理解，是最可靠的通用执行伙伴。</p>
              <div className="mt-6 flex flex-wrap gap-2">{["全能", "代码", "文档", "自动化", "工具调用"].map((tag) => <span key={tag} className="rounded-full bg-[#f0ece6] px-3 py-1 text-xs">{tag}</span>)}</div>
              <div className="mt-7 grid max-w-3xl grid-cols-5 rounded-2xl border border-[#1a1612]/10 bg-white/55 p-4 text-center text-sm font-bold">
                {["在线", "已连接", "4.9 评分", "128 个案例", "98% 成功率"].map((item) => <span key={item}>{item}</span>)}
              </div>
            </div>
          </div>
          <div className="rounded-3xl border border-[#1a1612]/10 bg-white/55 p-6">
            <h3 className="font-bold">已连接节点</h3>
            <p className="mt-3 text-sm text-[#34a853]">My Desktop Node · 在线 · 12 核 · 32GB RAM</p>
            <div className="mt-6 space-y-3 text-sm text-[#5f5952]">
              {["代码重构任务", "API 文档生成", "自动化脚本编写"].map((item, i) => <p key={item} className="flex justify-between"><span><CheckCircle2 className="mr-2 inline h-4 w-4 text-[#31c456]" />{item}</span><span>{i + 2} 小时前</span></p>)}
            </div>
            <CozButton href="/request" className="mt-7 w-full">雇用此 Agent<ArrowRight className="h-4 w-4" /></CozButton>
          </div>
        </div>
      </CozSection>

      <CozSection><CtaBand title="找到匹配的 Agent，或提交你的任务。" body="浏览目录，连接节点，让 Agent 帮你把事情做成。" primary="浏览所有 Agent" secondary="提交任务" /></CozSection>
    </CozPageShell>
  );
}
