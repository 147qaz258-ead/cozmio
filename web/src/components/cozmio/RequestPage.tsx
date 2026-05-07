"use client";

import Image from "next/image";
import { useState } from "react";
import { ArrowRight, Bell, BriefcaseBusiness, Lock, Send, Sparkles, ToggleRight } from "lucide-react";
import { cozmioAssets } from "@/lib/cozmio-assets";
import { requestOptions } from "@/lib/cozmio-data";
import { submitCozmioRequest } from "@/lib/request-submit";
import { CozBadge, CozButton, CozCard, CozIconOrb, CozSection } from "./Primitives";
import { FlowRail, PromiseStack } from "./VisualPanels";
import { CozPageShell } from "./Shell";
import { StageHero } from "./StageHero";

const fields = [
  ["项目类型", "projectType", "请选择项目类型", requestOptions.projectTypes],
  ["预算范围", "budget", "请选择预算范围", requestOptions.budgets],
  ["需要的 Agent 类型", "agentType", "选择或搜索所需 Agent 类型（可多选）", requestOptions.agentTypes],
  ["交付时间", "timeline", "请选择期望的交付时间", requestOptions.timelines],
] as const;

export function RequestPage() {
  const [status, setStatus] = useState<"idle" | "submitting" | "success" | "error">("idle");
  const [message, setMessage] = useState("");
  const [taskLength, setTaskLength] = useState(0);

  async function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    setStatus("submitting");
    setMessage("");

    try {
      await submitCozmioRequest({
        task: String(data.get("task") || ""),
        projectType: String(data.get("projectType") || ""),
        budget: String(data.get("budget") || ""),
        agentType: String(data.get("agentType") || ""),
        timeline: String(data.get("timeline") || ""),
        publicCase: data.get("publicCase") === "on",
        contact: String(data.get("contact") || ""),
      });
      setStatus("success");
      setMessage("已收到你的请求。Cozmio 会尽快评估并联系你。");
      form.reset();
      setTaskLength(0);
    } catch (error) {
      setStatus("error");
      setMessage(error instanceof Error ? error.message : "提交失败，请稍后再试");
    }
  }

  return (
    <CozPageShell>
      <StageHero
        eyebrow={<CozBadge>REQUEST</CozBadge>}
        title={<>告诉 Cozmio,<br />你想让 Agent 帮你做什么。</>}
        body="提交你的任务，网络将为你匹配最适配的 Builder 与 Agent。当需要本地能力时，Desktop Node 也会加入协作。"
        visualSrc={cozmioAssets.requestHero}
        visualAlt="Cozmio Task Request 网络"
        visualClassName="object-[66%_center]"
        pills={[
          ["智能匹配最佳资源", Sparkles],
          ["端到端安全可控", Lock],
          ["全程透明可追踪", Bell],
        ].map(([text, Icon]) => <CozBadge key={text as string}><Icon className="h-4 w-4" />{text as string}</CozBadge>)}
      />

      <CozSection className="relative z-10 -mt-24 grid items-start gap-8 lg:grid-cols-[1.45fr_.82fr]">
        <CozCard className="p-10">
          <div className="mb-8 flex items-center justify-between">
            <h2 className="text-3xl font-bold">创建请求</h2>
            <span className="text-xs font-bold text-[#7d766f]">* 为必填项</span>
          </div>
          <form onSubmit={submit} className="space-y-5">
            <label className="grid gap-4 md:grid-cols-[170px_1fr]">
              <span className="flex items-start gap-3 font-bold"><span className="grid h-7 w-7 place-items-center rounded-full border border-[#efb767] text-sm text-[#d48b22]">1</span>你想做什么 *</span>
              <div>
                <textarea
                  name="task"
                  required
                  maxLength={1000}
                  rows={5}
                  onChange={(event) => setTaskLength(event.target.value.length)}
                  className="coz-input resize-none"
                  placeholder="请描述你的任务目标、期望的成果、背景信息与关键要求..."
                />
                <div className="-mt-8 pr-4 text-right text-xs text-[#aaa198]">{taskLength}/1000</div>
              </div>
            </label>
            {fields.map(([label, name, placeholder, options], index) => (
              <label key={name} className="grid gap-4 md:grid-cols-[170px_1fr]">
                <span className="flex items-center gap-3 font-bold"><span className="grid h-7 w-7 place-items-center rounded-full border border-[#efb767] text-sm text-[#d48b22]">{index + 2}</span>{label} *</span>
                <select name={name} required className="coz-input text-[#5f5952]">
                  <option value="">{placeholder}</option>
                  {options.map((option) => <option key={option} value={option}>{option}</option>)}
                </select>
              </label>
            ))}
            <label className="grid gap-4 md:grid-cols-[170px_1fr]">
              <span className="flex items-center gap-3 font-bold"><span className="grid h-7 w-7 place-items-center rounded-full border border-[#efb767] text-sm text-[#d48b22]">6</span>是否允许公开为案例</span>
              <span className="flex items-center justify-end gap-3 rounded-2xl px-2 py-2 text-sm font-bold text-[#5f5952]"><input name="publicCase" type="checkbox" defaultChecked className="sr-only" /><ToggleRight className="h-10 w-10 text-[#31c456]" />允许</span>
            </label>
            <label className="grid gap-4 md:grid-cols-[170px_1fr]">
              <span className="flex items-center gap-3 font-bold"><span className="grid h-7 w-7 place-items-center rounded-full border border-[#efb767] text-sm text-[#d48b22]">7</span>联系方式 *</span>
              <input name="contact" required className="coz-input" placeholder="邮箱 / 手机号 / 其他联系方式" />
            </label>
            <div className="flex flex-wrap items-center gap-5 pt-5">
              <CozButton type="submit" className="min-w-36">{status === "submitting" ? "提交中..." : "提交请求"}<ArrowRight className="h-4 w-4" /></CozButton>
              <p className="flex items-center gap-2 text-sm text-[#756e66]"><Lock className="h-4 w-4" />你的信息将被严格保密，仅用于任务匹配与协作</p>
            </div>
            {message ? (
              <div className={`rounded-2xl px-5 py-4 text-sm font-semibold ${status === "success" ? "bg-[#e7f8e8] text-[#25863a]" : "bg-[#fff0e2] text-[#b05a19]"}`}>
                {message}
              </div>
            ) : null}
          </form>
        </CozCard>

        <div className="space-y-5">
          <CozCard className="p-9">
            <h2 className="text-3xl font-bold">提交流程</h2>
            <div className="mt-7"><FlowRail mode="request" /></div>
          </CozCard>
          <CozCard className="p-4"><PromiseStack /></CozCard>
        </div>
      </CozSection>

      <CozSection className="mt-8 grid gap-7 lg:grid-cols-[.7fr_1.3fr]">
        <CozCard className="p-8">
          <h2 className="text-3xl font-bold">适合做什么</h2>
          <div className="mt-7 flex flex-wrap gap-3">
            {["软件开发", "数据分析", "自动化脚本", "设计创意", "研究报告", "内容生成", "产品原型", "更多场景..."].map((tag) => <span key={tag} className="rounded-full bg-[#f0ece6] px-4 py-3 text-sm font-semibold text-[#6d665f]">{tag}</span>)}
          </div>
        </CozCard>
        <CozCard className="p-8">
          <div className="mb-5 flex justify-between"><h2 className="text-2xl font-bold">近期请求案例</h2><span className="text-sm font-bold">查看全部案例 →</span></div>
          <div className="grid gap-4 md:grid-cols-3">
            {["电商数据分析看板", "企业内部知识库搭建", "自动化运营脚本开发"].map((title, i) => <div key={title} className="rounded-2xl border border-[#1a1612]/10 bg-white/55 p-5"><h3 className="font-bold">{title}</h3><p className="mt-3 text-sm text-[#756e66]">预算：¥{i + 1}k - ¥{i + 3}k</p><p className="mt-2 text-sm text-[#756e66]">交付时间：{i + 2} 天</p><span className="mt-4 inline-block rounded-full bg-[#e7f8e8] px-3 py-1 text-xs font-bold text-[#34a853]">已交付</span></div>)}
          </div>
        </CozCard>
      </CozSection>

      <CozSection className="mt-10">
        <div className="coz-card p-8">
          <h2 className="text-3xl font-bold">不知道怎么描述？试试这些快速示例</h2>
          <div className="mt-7 grid gap-5 md:grid-cols-3">
            {[
              ["开发一个数据看板", "我需要一个销售数据看板，支持多维度分析和可视化展示。", BriefcaseBusiness],
              ["搭建 AI 客服机器人", "基于企业知识库，搭建一个智能客服机器人，自动回答常见问题。", Sparkles],
              ["自动化报表生成", "每天自动从多个数据源获取数据，生成并发送日报/周报。", Send],
            ].map(([title, body, Icon]) => <CozCard key={title as string} className="p-7"><CozIconOrb icon={Icon as typeof Sparkles} tone="violet" /><h3 className="mt-5 text-xl font-bold">{title as string}</h3><p className="mt-3 min-h-16 text-sm leading-6 text-[#706961]">{body as string}</p><CozButton href="/request" variant="light" className="mt-4 h-10 px-5 text-sm">使用示例<ArrowRight className="h-4 w-4" /></CozButton></CozCard>)}
          </div>
        </div>
      </CozSection>

      <CozSection className="mt-10">
        <div className="coz-soft-band flex items-center justify-between gap-8 px-10 py-9">
          <div><h2 className="text-4xl font-bold">准备好了吗？现在就提交你的请求。</h2><p className="mt-4 text-[#6d665f]">让 Cozmio 网络为你匹配最佳资源，开启高效协作。</p><CozButton href="#top" className="mt-7">立即提交请求<ArrowRight className="h-4 w-4" /></CozButton></div>
          <Image src={cozmioAssets.ctaCore} alt="提交请求发光节点" width={420} height={200} className="hidden w-[360px] object-contain md:block" />
        </div>
      </CozSection>
    </CozPageShell>
  );
}
