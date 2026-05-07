# Cozmio Agent Build Network 实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：将 `web/` 从 Pulseclaw 桌面上下文官网重做为 Cozmio Agent Build Network，包含首页、Agents、Projects、Cases、Desktop、Request 六个可导航页面。

**架构思路**：保留现有 Next.js App Router、Tailwind 4、布局组件和静态导出模式，替换品牌配置与主页叙事，新增集中 mock data 与网络入口页面。视觉上用轻量 HTML/CSS 实现 Agent Relay Network，不引入重型 3D 依赖。

**技术栈**：Next.js 16.2.2、React 19.2.4、TypeScript、Tailwind CSS 4、lucide-react、现有 shadcn-style Button/Card 基础组件。

---

## 产品类型

传统软件实现型。

理由：本次工作主要是页面、组件、导航、mock data 和表单 UI 改造，核心风险是构建、路由、响应式布局和内容一致性，不涉及模型输出 judgment、abstain 或 grounding 质量验证。

## 文件结构

- 修改：`D:\C_Projects\Agent\cozmio\web\src\lib\site-config.ts`
  - 职责：统一 Cozmio 品牌、导航、页脚链接、站点 metadata 文案。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\lib\network-data.ts`
  - 职责：集中维护 agents、projects、cases、desktop capabilities、request options mock data。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\page.tsx`
  - 职责：组合新首页区块。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\components\HeroSection.tsx`
  - 职责：替换首屏文案和 CTA，承载 Agent Relay Network 视觉。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\AgentRelayVisual.tsx`
  - 职责：实现 Task Core、Agent Nodes、Result Flow、Memory/Cases Sink 的轻量视觉。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\NetworkPreview.tsx`
  - 职责：首页展示 agent preview、desktop preview、case preview、request CTA。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\components\layout\Header.tsx`
  - 职责：品牌改为 Cozmio，主按钮改为 Submit a Task / 提交任务，保留响应式导航。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\components\layout\Footer.tsx`
  - 职责：页脚改为 Cozmio 网络入口链接。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\agents\page.tsx`
  - 职责：agent directory 页面。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\projects\page.tsx`
  - 职责：project/request listing 页面。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\cases\page.tsx`
  - 职责：delivery cases 页面。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\desktop\page.tsx`
  - 职责：Cozmio Desktop Node 页面。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\request\page.tsx`
  - 职责：任务提交页面与静态表单 UI。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\globals.css`
  - 职责：仅在必要时补充 relay visual 动画和移除明显 Pulseclaw 注释，不做大规模重构。

## 任务 1：品牌配置和 mock data

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\lib\site-config.ts`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\lib\network-data.ts`

- [ ] **步骤 1：更新站点配置**

将 `siteConfig` 改为 Cozmio 品牌：

```ts
export const siteConfig = {
  name: "Cozmio",
  shortName: "Cozmio",
  tagline: "Agent Build Network",
  title: "Cozmio - 带本地节点的 Agent 构建网络",
  description:
    "Cozmio lets agents, builders, projects, and tasks discover each other, collaborate, and deliver work through a network with local desktop nodes.",
  siteUrl: process.env.NEXT_PUBLIC_SITE_URL?.replace(/\/$/, "") ?? "https://cozmio.net",
  email: "jinhongw840@gmail.com",
  desktopNote:
    "Cozmio Desktop Node turns a user's computer into a local node for real projects, long-term memory, and executor tools.",
  links: {
    github: "https://github.com/147qaz258-ead/Pulseclaw",
    x: "https://x.com/wjnhng419090",
    wechat: "https://mp.weixin.qq.com/s/JRRaF3-xg345A6ey-poelw",
    email: "mailto:jinhongw840@gmail.com",
  },
  navItems: [
    { label: "Agents", href: "/agents" },
    { label: "Projects", href: "/projects" },
    { label: "Cases", href: "/cases" },
    { label: "Desktop", href: "/desktop" },
    { label: "Request", href: "/request" },
  ],
} as const;
```

- [ ] **步骤 2：更新页脚分组**

将 `footerGroups` 改为 Network、Build、Learn、Legal 四组，链接到 `/agents`、`/projects`、`/cases`、`/desktop`、`/request`，保留 `/blog`、`/privacy`、`/terms`、邮箱和 GitHub。

- [ ] **步骤 3：创建 mock data 文件**

在 `src/lib/network-data.ts` 中导出：

```ts
export const agents = [
  {
    name: "Cozmio Desktop Agent",
    status: "Experimental",
    node: "Desktop Node connected",
    capabilities: ["Local project memory", "Claude Code execution", "Desktop state capture"],
    taskTypes: ["Landing page iteration", "Local codebase work", "Delivery record"],
    cases: "1 internal case",
  },
  {
    name: "Claude Code Executor",
    status: "Available",
    node: "Executor card ready",
    capabilities: ["Code edits", "Test runs", "Pull request preparation"],
    taskTypes: ["Bug fix", "Feature implementation", "Refactor"],
    cases: "3 sample deliveries",
  },
  {
    name: "Research Agent",
    status: "Available",
    node: "Cloud agent",
    capabilities: ["Market scan", "Source synthesis", "Brief writing"],
    taskTypes: ["Product research", "Competitor notes", "Decision memo"],
    cases: "2 sample briefs",
  },
  {
    name: "Landing Page Agent",
    status: "Busy",
    node: "Builder profile",
    capabilities: ["Message hierarchy", "Page sections", "Responsive UI"],
    taskTypes: ["Homepage rewrite", "Case page", "Request funnel"],
    cases: "4 mock cases",
  },
] as const;
```

同文件补充 `projects`、`cases`、`desktopCapabilities`、`requestOptions`，每个数组至少 3 项，全部使用真实可展示文案，不使用空字符串。

- [ ] **步骤 4：运行类型检查构建入口**

执行命令：`npm run build`

预期结果：如果其他旧页面没有阻塞，Next 构建完成；如果旧页面暴露无关历史错误，记录错误文件和错误消息，先完成本任务涉及文件的 TypeScript 导入修正。

## 任务 2：首页首屏和 Relay Visual

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\page.tsx`
- 修改：`D:\C_Projects\Agent\cozmio\web\src\components\HeroSection.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\AgentRelayVisual.tsx`
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\globals.css`

- [ ] **步骤 1：创建 AgentRelayVisual 组件**

实现结构：

```tsx
export function AgentRelayVisual() {
  return (
    <div className="relative min-h-[420px] overflow-hidden rounded-[2rem] border border-white/12 bg-primary-text p-6 text-white shadow-[0_30px_90px_rgba(45,42,38,0.22)]">
      <div className="absolute inset-0 relay-grid opacity-35" />
      <div className="relative mx-auto flex h-[360px] max-w-[520px] items-center justify-center">
        <div className="absolute rounded-full border border-mist-blue/35 bg-white/10 px-6 py-5 text-center backdrop-blur">
          <div className="text-xs font-semibold uppercase tracking-[0.16em] text-mist-blue">Build Request</div>
          <div className="mt-2 text-xl font-semibold">Task Core</div>
        </div>
        {/* Place five nodes around the core and connect them with absolute lines. */}
      </div>
    </div>
  );
}
```

节点文字必须包含 Claude Code、Codex、Gemini、Desktop Node、Memory / Cases。

- [ ] **步骤 2：补充 CSS 动画**

在 `globals.css` 的 utilities 区添加 `.relay-grid`、`.relay-token`，使用现有 `ambient-pulse` 和 `story-flow-line` 风格，不引入外部动画库。

- [ ] **步骤 3：替换 HeroSection 文案**

Hero 内容必须包含：

- kicker：`Agent Build Network`
- H1：`带着你的 Agent，帮别人把东西做出来。`
- subtitle：规格中的中文副标题
- primary CTA：`提交任务` 指向 `/request`
- secondary CTA：`查看桌面节点` 指向 `/desktop`
- supporting line：`Agent 负责执行，Cozmio 负责接力、记录和交付。`

- [ ] **步骤 4：替换 page.tsx 组合**

首页只保留 `Header`、`HeroSection`、新的 `NetworkPreview`、`Footer`，移除旧的 `CapabilityCards`、`HowItWorks`、`CTASection` 组合。

- [ ] **步骤 5：运行 lint**

执行命令：`npm run lint`

预期结果：无新增 lint 错误；如果 lint 报旧文件问题，记录错误文件，并修正本任务新增/修改文件中的错误。

## 任务 3：首页 NetworkPreview

**涉及文件**：

- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\NetworkPreview.tsx`
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\page.tsx`

- [ ] **步骤 1：创建 NetworkPreview**

组件从 `network-data.ts` 读取 mock data，渲染四个首页区块：

- Agent Network：展示前 4 个 agent cards。
- Desktop Node：展示 desktop capabilities。
- Cases：展示第一条 case。
- Request：显示 `/request` CTA。

- [ ] **步骤 2：保持区块视觉克制**

使用 full-width section + constrained inner content。卡片只用于重复条目，不把整段 section 包在大卡片里。卡片圆角使用 `rounded-xl` 或 `rounded-2xl`，不再加更大的装饰圆角。

- [ ] **步骤 3：检查移动端布局**

组件所有 grid 必须默认单列，`md:` 或 `lg:` 再变多列。长标题不能放进固定宽度按钮。

- [ ] **步骤 4：运行构建**

执行命令：`npm run build`

预期结果：构建成功，首页静态生成成功。

## 任务 4：Header 和 Footer 改造

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\components\layout\Header.tsx`
- 修改：`D:\C_Projects\Agent\cozmio\web\src\components\layout\Footer.tsx`

- [ ] **步骤 1：修改 Header 品牌**

把左侧标识从 `P / Pulseclaw / Context Before Prompt` 改为：

- 图标字母：`C`
- 品牌名：`Cozmio`
- tagline：`Agent Build Network`

- [ ] **步骤 2：修改 Header 右侧按钮**

移除 `Run Demo` 主按钮，改为 `/request` 的 `Submit a Task`。GitHub 链接可保留为次级外链。

- [ ] **步骤 3：移除桌面优先旧状态条**

把 `Desktop-first public site` 改为 `Local nodes in private beta`。

- [ ] **步骤 4：修改 Footer 文案**

页脚 copyright 改为 `© 2026 Cozmio. All rights reserved.`，说明文案使用 `siteConfig.description` 和 `siteConfig.desktopNote`。

- [ ] **步骤 5：运行 lint**

执行命令：`npm run lint`

预期结果：Header/Footer 无 lint 错误。

## 任务 5：新增 Agents、Projects、Cases 页面

**涉及文件**：

- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\agents\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\projects\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\cases\page.tsx`

- [ ] **步骤 1：创建 `/agents`**

页面必须包含：

- Metadata title 和 description。
- Header/Footer。
- 页面标题 `Agents that can be discovered, hired, and connected.`
- Cards 展示 `agents` 全量数据。
- 每张卡包含 status、node、capabilities、taskTypes、cases。

- [ ] **步骤 2：创建 `/projects`**

页面必须包含：

- Metadata title 和 description。
- Header/Footer。
- 页面标题 `Projects waiting for agents and builders.`
- Cards 展示 `projects` 数据。
- 每张卡包含 outcome、needed、budget/status、public case permission。

- [ ] **步骤 3：创建 `/cases`**

页面必须包含：

- Metadata title 和 description。
- Header/Footer。
- 页面标题 `Delivery cases that leave proof of work.`
- Cards 展示 `cases` 数据。
- 第一条 case 标题必须是 `Cozmio 如何用桌面节点推进自己的 landing page 改版`。

- [ ] **步骤 4：运行构建**

执行命令：`npm run build`

预期结果：`/agents`、`/projects`、`/cases` 均被 Next 构建为有效路由。

## 任务 6：新增 Desktop 和 Request 页面

**涉及文件**：

- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\desktop\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\app\request\page.tsx`

- [ ] **步骤 1：创建 `/desktop`**

页面必须包含：

- Metadata title 和 description。
- Header/Footer。
- H1：`把你的电脑变成 Agent 网络里的本地节点。`
- Four capability blocks：真实项目、长期记忆、执行端连接、交付记录。
- Sample node JSON block：

```json
{
  "node_name": "jin-hong-local-node",
  "capabilities": ["claude-code", "local-memory", "desktop-capture", "custom-command"],
  "status": "private-beta"
}
```

- [ ] **步骤 2：创建 `/request`**

页面必须包含静态表单 UI：

- `我想做什么？` textarea。
- `预算 / 是否免费试用` select 或 radio。
- `需要什么 agent？` select。
- `是否允许公开案例？` checkbox or radio。
- `联系方式` input。
- Submit button text：`提交任务`。

表单第一版不需要后端提交；按钮可使用 `type="button"`，旁边显示 `Private beta requests are reviewed manually.`。

- [ ] **步骤 3：确保表单可访问**

每个 input/select/textarea 必须有 `<label>`，不能只靠 placeholder。

- [ ] **步骤 4：运行 lint 和 build**

执行命令：

```bash
npm run lint
npm run build
```

预期结果：新增页面无 lint 错误，构建成功。

## 任务 7：最终验证和视觉检查

**涉及文件**：

- 检查全部本次修改文件。

- [ ] **步骤 1：运行完整构建**

执行命令：`npm run build`

预期结果：构建成功，静态页面输出成功。

- [ ] **步骤 2：启动本地开发服务器**

执行命令：`npm run dev`

预期结果：Next dev server 输出可访问 URL，例如 `http://localhost:3000`。

- [ ] **步骤 3：浏览器检查关键页面**

用浏览器打开：

- `http://localhost:3000/`
- `http://localhost:3000/agents`
- `http://localhost:3000/projects`
- `http://localhost:3000/cases`
- `http://localhost:3000/desktop`
- `http://localhost:3000/request`

验收标准：

- Header 导航全部可点击。
- 首页首屏主标题为 `带着你的 Agent，帮别人把东西做出来。`
- 首页主视觉不是机器人，而是任务核心、agent 节点、流转线和 cases/memory。
- Request 页面表单字段完整。
- Desktop 页面清楚说明 Cozmio Desktop Node。
- 移动端宽度下没有按钮文字溢出、卡片文本重叠、导航遮挡。

- [ ] **步骤 4：搜索旧主品牌残留**

执行 PowerShell 命令：

```powershell
Select-String -Path 'src\app\page.tsx','src\components\HeroSection.tsx','src\components\layout\Header.tsx','src\components\layout\Footer.tsx','src\lib\site-config.ts' -Pattern 'Pulseclaw|Context Before Prompt|Run Demo|上下文|证据链'
```

预期结果：本次重做的首页、导航、页脚和 site config 中不再出现旧主叙事词。旧 blog/demo 页面可保留历史内容。

- [ ] **步骤 5：整理结果说明**

最终说明必须列出：

- 修改了哪些页面。
- 验证命令及结果。
- 本地 dev server URL。
- 如果有旧页面历史内容保留，说明其不在主导航中。
