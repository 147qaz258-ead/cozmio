# Cozmio 截图级官网复刻实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：基于用户提供的 5 张 Cozmio 页面设计稿，尽可能一比一实现 Next.js / React / Tailwind 网站：首页、Desktop Node、Agents、Cases、Request 五个核心页面必须与截图视觉高度一致。

**架构思路**：保留现有 `web/` Next.js App Router 与 Tailwind 4，不引入 WebGL、react-three-fiber、drei 或不稳定依赖。先建立统一的截图级设计系统与页面数据，再按截图逐页复刻；复杂 3D 视觉使用生成 bitmap 资产，React/Tailwind 负责版式、文字、卡片、表单、状态和响应式。当前实现中的普通 SaaS 卡片感组件将被替换为截图中的“浅米色空间背景 + 玻璃拟物卡片 + 暖金光线 + 高级暗色节点面板”的统一系统。

**技术栈**：Next.js 16.2.2、React 19.2.4、TypeScript、Tailwind CSS 4、lucide-react、Next static export、生成 bitmap 资产、Playwright/Chrome DevTools 视觉验收。

---

## 最高执行纪律

1. 首页未通过用户视觉验收前，禁止继续实现其他页面。
2. 第一轮只做首页 `/` 和提交入口 `/request` 的最小可用版本。
3. 首页目标不是"有内容"，而是"第一眼惊艳、可信、像一个高端产品"。
4. Request 页面必须能真实收集用户需求，不能只是静态表单。
5. Codex 不允许重新设计，只能复刻设计稿。
6. 复杂 3D 视觉使用预渲染图片资产，文字标签使用 HTML/CSS 叠加。
7. 第一版只做中文。中 / EN 只保留视觉，不实现完整双语。
8. 每完成一个页面必须截图验收，通过后再进入下一页。

**商业目标**：用户看了首页，愿意点提交任务。不是"5个页面齐全"，是"用户看到首页 → 想提交任务"。

---

## 产品类型

传统软件实现型。

本次核心风险是页面复刻精度、样式一致性、静态导出、图片资源质量、响应式不溢出、双语切换和构建稳定性，不涉及模型输出 judgment、abstain 或 grounding 质量验证。

## 视觉真相与页面范围

用户提供的 5 张截图是唯一视觉真相：

1. 首页：Hero + 三个入口卡片 + Desktop 深色面板 + 真实案例 + CTA + Footer。
2. Desktop Node 页面：左侧强标题 + 右侧深色节点状态面板 + 本地执行/长期记忆/连接执行端 + 流程 + 执行端 + 数据规则 + CTA + Footer。
3. Agents 页面：左侧标题 + 右侧 3D agent core 视觉 + 搜索筛选 + 指标条 + Agent 网格 + 推荐 Agent + Builder/节点提供者 + CTA + Footer。
4. Cases 页面：左侧标题 + 右侧精选案例 + 分类筛选 + 6 个案例网格 + 交付轨迹 + testimonial + CTA + Footer。
5. Request 页面：左侧标题 + 右侧 task request 3D 视觉 + 大表单 + 提交流程侧栏 + 适合做什么 + 近期请求案例 + 快速示例 + CTA + Footer。

不可自由发挥：

- 不重新排版。
- 不更换截图里的核心文案。
- 不把页面做成普通 SaaS 模板。
- 不把“上下文、证据链、runtime”作为主语言。
- 不使用随机装饰图标堆砌。
- 不用 WebGL 或 3D 运行时。

## 设计系统目标

### 页面底层

- 背景为暖白/米白，不是纯白。
- 全站背景带非常轻的暖金光线、柔和径向高光、低对比噪声感。
- 首页和 Request 顶部允许使用大范围生成主视觉图作为底层背景的一部分，形成“整张作品”而非左右拼接。

### 页面中层

- Header、Hero、主内容区、CTA、Footer 统一在同一视觉语言里。
- 主容器宽度接近截图：桌面端 `max-width: 1440px`，页面内主要内容左右边距约 `40px` 到 `56px`。
- Section 之间有充足垂直呼吸，不能每段都是等距卡片网格。

### 页面上层

- 玻璃拟物卡片：白色半透明、细边框、柔和内阴影、低饱和投影。
- 暗色节点面板：接近黑蓝灰，绿色在线状态，高亮圆环，清晰模块分区。
- Badge / Button / Card / Footer 完全统一。
- 图标只服务信息：搜索、电脑、文件夹、数据库、闪电、锁、安全，不堆装饰。

## 真实 3D 资源策略

第一阶段不使用实时 3D / WebGL / Spline embed / GLB。

第一阶段使用预渲染 3D bitmap 资产，目标是视觉稳定、上线速度和截图级复刻。

第二阶段，当首页视觉和转化路径通过验证后，再将首页 Hero 的 bitmap 主视觉替换为：

- Spline 场景；
- 或 GLB + Three.js；
- 或可交互 Agent Relay Network。

第二阶段只替换 Hero 主视觉，不全站引入实时 3D。

这就是正确顺序：**先高级稳定 → 再交互升级**。

## 核心产品卖点

全站必须持续表达：

1. Agent 可以被发现、被雇用、被连接；
2. 桌面端让用户电脑成为本地节点；
3. **越用，系统越理解你**；
4. 每次交付都会沉淀为下一次协作的资产。

这比"长期记忆"更人话，也更值钱。

## 文件结构

- 修改：`D:\C_Projects\Agent\cozmio\web\next.config.ts`
  - 维持 `output: "export"` 与 `images.unoptimized: true`，保证生成图片静态导出可用。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\globals.css`
  - 建立 Cozmio 截图级设计 token、背景、卡片、按钮、badge、光线、深色节点面板、页面 spacing。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\lib\cozmio-copy.ts`
  - 保存五个页面截图文案、数据、筛选项、统计、表单字段。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\lib\cozmio-assets.ts`
  - 集中定义生成图片资产路径和 alt 文案。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Shell.tsx`
  - 统一页面壳、Header、Footer。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Primitives.tsx`
  - Button、Badge、GlassCard、SectionBand、StatPill、IconOrb、MiniAgentCard、FooterLinkGroup。
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\VisualPanels.tsx`
  - 复刻截图里的 3D 图像容器、深色 Desktop 面板、案例缩略图、流程轨迹、请求流程侧栏。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\page.tsx`
  - 首页按截图 1 重做。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\desktop\page.tsx`
  - Desktop Node 页面按截图 2 重做。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\agents\page.tsx`
  - Agents 页面按截图 3 重做。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\cases\page.tsx`
  - Cases 页面按截图 4 重做。
- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\request\page.tsx`
  - Request 页面按截图 5 重做。
- 创建目录：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\`
  - 保存生成的主视觉和局部视觉资产。
- 保留但不作为主导航：`D:\C_Projects\Agent\cozmio\web\src\app\demo`、`progress`、`blog`、`about`、`contact`。

## 资产生成清单

所有图片必须保存到 `web/public/images/cozmio/`，原始生成图保留在 `$CODEX_HOME/generated_images`。

### 资产 A：首页 Hero 主视觉

文件：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\home-task-core-hero.png`

用途：首页右侧大主视觉，复刻截图 1 的 Task Core 网络。

生成提示词：

```text
Use case: ads-marketing
Asset type: premium website hero visual
Primary request: Create a high-end 3D product hero for Cozmio Agent Build Network. A glowing transparent cube labeled visually as a central task core without readable text, sitting on a polished circular platform, surrounded by floating glass agent cards representing Claude Code, Codex, Gemini, Desktop Node, Cases, and Memory. Warm champagne light trails connect all nodes. Premium AI product website look, trustworthy, refined, not generic SaaS.
Scene/backdrop: warm ivory studio space, soft glass, circular platform, subtle network rings, luminous flowing lines.
Style/medium: cinematic 3D render, high-end product launch visual, frosted glass, chrome, warm gold, mist blue energy accents.
Composition/framing: wide landscape, central task core slightly right of center, enough negative space on left, no hard crop, layered depth.
Lighting/mood: soft warm bloom, white-gold highlights, subtle cyan connection lines, premium and calm.
Color palette: ivory, champagne gold, charcoal, mist blue, soft green status dots.
Text: no readable text, no logos, no watermark.
Constraints: no humanoid robot, no cyberpunk darkness, no random UI text, no clutter, no distorted symbols.
```

### 资产 B：Desktop 深色节点面板背景

文件：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\desktop-node-panel.png`

用途：首页 Desktop 区域与 Desktop 页面右侧大面板。

生成提示词：

```text
Use case: ui-mockup
Asset type: dark product interface panel
Primary request: Create a high-end dark desktop node dashboard mockup for Cozmio Desktop Node. Include a left sidebar, online node status, green trust ring, memory metric, executor tiles for Claude Code, Codex, Gemini, Custom CLI, recent activity rows, and privacy/security status cards. No readable small text needed; layout should match a premium desktop app screenshot.
Scene/backdrop: dark glass software interface floating on warm ivory website background.
Style/medium: polished 3D UI mockup, realistic glass, soft shadows, deep black-blue panel, green status accent.
Composition/framing: wide rectangular panel, front-facing slight perspective, clean margins.
Lighting/mood: premium, secure, calm, high contrast.
Text: no readable text, no watermark.
Constraints: no busy random widgets, no fake code blocks, no neon cyberpunk, no charts that dominate.
```

### 资产 C：Agents 页 Core 视觉

文件：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\agents-core.png`

用途：Agents 页右上主视觉。

生成提示词：

```text
Use case: ads-marketing
Asset type: agent directory hero visual
Primary request: Create a luminous 3D agent directory core for Cozmio. A transparent cube with a friendly abstract agent glyph inside, floating above a platform, surrounded by small glass icon tiles for code, search, lightning, design, and user profile. Premium warm ivory background and soft network rings.
Style/medium: refined 3D product render, frosted glass, champagne light, mist blue and green accents.
Composition/framing: wide hero crop, visual on right side, clean left negative space.
Text: no readable text, no logo, no watermark.
Constraints: no robot, no cartoon mascot, no dark cyberpunk.
```

### 资产 D：Cases 页案例缩略图组

文件：

- `D:\C_Projects\Agent\cozmio\web\public\images\cozmio\case-saas-dashboard.png`
- `D:\C_Projects\Agent\cozmio\web\public\images\cozmio\case-data-dashboard.png`
- `D:\C_Projects\Agent\cozmio\web\public\images\cozmio\case-automation-flow.png`
- `D:\C_Projects\Agent\cozmio\web\public\images\cozmio\case-design-system.png`
- `D:\C_Projects\Agent\cozmio\web\public\images\cozmio\case-api-platform.png`
- `D:\C_Projects\Agent\cozmio\web\public\images\cozmio\case-support-automation.png`

用途：Cases 页面 6 个案例卡片缩略图。

生成提示词模板：

```text
Use case: ui-mockup
Asset type: case study thumbnail
Primary request: Create a premium product case thumbnail for Cozmio: <case theme>. It should look like a polished software screenshot inside a soft rounded card, with no readable small text.
Scene/backdrop: warm ivory or dark glass product environment depending on theme.
Style/medium: high-end UI mockup, subtle depth, refined spacing.
Composition/framing: 16:9 crop, centered software panel, no text required.
Constraints: no logos, no watermark, no random text, no cheap SaaS template feel.
```

主题依次替换为：

- multi-tenant SaaS admin platform
- sales analytics and forecasting dashboard
- automated content generation pipeline
- brand website and design system workspace
- developer API platform
- customer support automation assistant

### 资产 E：Request 页 Task Request 主视觉

文件：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\request-task-core.png`

用途：Request 页右侧 hero 视觉，复刻截图 5。

生成提示词：

```text
Use case: ads-marketing
Asset type: request page hero visual
Primary request: Create a premium 3D visual for submitting a task to Cozmio. A glowing task request cube on a circular platform, connected to floating glass cards for Claude Code, Codex, Gemini, Desktop Node, Cases, and Memory. The visual should feel like a request entering an agent network.
Scene/backdrop: warm ivory studio background, champagne light trails, soft glass.
Style/medium: high-end 3D product render, refined and trustworthy.
Composition/framing: wide landscape, central cube slightly right, clean left side.
Text: no readable text, no watermark.
Constraints: no robot, no random UI text, no clutter.
```

### 资产 F：CTA 小型发光核心

文件：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\cta-core.png`

用途：首页、Agents、Cases、Request、Desktop 底部 CTA 右侧小视觉。

生成提示词：

```text
Use case: ads-marketing
Asset type: small CTA visual
Primary request: Create a small glowing transparent cube core with warm gold light trails and small floating agent icons, for a premium Cozmio website CTA band.
Style/medium: refined 3D render, transparent glass, champagne glow, warm ivory background.
Composition/framing: wide crop with visual on right side and clean left side.
Text: no text, no logo, no watermark.
Constraints: no robot, no clutter.
```

## 任务实施顺序（分阶段）

计划顺序已调整为优先保证首页和 Request 页面的最小可用版本：

**第一阶段（必须先完成）**：
- 任务 1A：设计系统最小版（token + shell + primitives，仅支撑首页和 request）
- 任务 1B：首页和 Request 所需资产（资产 A + 资产 E + 资产 F）
- 任务 3：首页按截图 1 复刻
- 任务 7：Request 页面按截图 5 复刻（必须能真实提交）

**第二阶段（首页验收通过后）**：
- 任务 4：Desktop 页面
- 任务 5：Agents 页面
- 任务 6：Cases 页面

**第三阶段（细节收尾）**：
- 任务 8：双语策略收敛
- 任务 9：响应式与稳定性
- 任务 10：截图级视觉验收

> 每完成一个页面必须截图验收，通过后再进入下一页。没有例外。

---

## 任务 1A：设计系统最小版（仅支撑首页 + Request）

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\globals.css`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Primitives.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Shell.tsx`
- 修改：`D:\C_Projects\Agent\cozmio\web\src\lib\site-config.ts`

- [ ] **步骤 1：定义全站色彩和背景**

在 `globals.css` 中调整 Cozmio token：

```css
--color-coz-bg: #f7f3ed;
--color-coz-panel: rgba(255,255,255,0.72);
--color-coz-panel-strong: rgba(255,255,255,0.86);
--color-coz-border: rgba(28,24,20,0.08);
--color-coz-text: #141414;
--color-coz-muted: #6f6a64;
--color-coz-gold: #f5b544;
--color-coz-green: #31c456;
--color-coz-blue: #6aa7ff;
--color-coz-purple: #8a6cff;
--color-coz-orange: #ff8b3d;
--color-coz-dark: #11161b;
```

`body` 背景必须改为截图级暖米色与柔光：

```css
background:
  radial-gradient(circle at 18% 8%, rgba(255,255,255,0.95), transparent 32%),
  radial-gradient(circle at 82% 12%, rgba(246,210,155,0.22), transparent 34%),
  linear-gradient(180deg, #f8f5ef 0%, #f4efe8 100%);
```

- [ ] **步骤 2：添加通用视觉类**

在 `globals.css` 添加：

```css
.coz-shell { max-width: 1440px; margin-inline: auto; padding-inline: 36px; }
.coz-glass { background: rgba(255,255,255,.72); border: 1px solid rgba(26,22,18,.08); box-shadow: 0 24px 70px rgba(44,34,22,.08), inset 0 1px 0 rgba(255,255,255,.75); backdrop-filter: blur(18px); }
.coz-card { border-radius: 18px; background: rgba(255,255,255,.74); border: 1px solid rgba(26,22,18,.08); box-shadow: 0 18px 48px rgba(44,34,22,.06); }
.coz-dark-panel { background: linear-gradient(145deg,#11171d,#080c10); border: 1px solid rgba(255,255,255,.08); box-shadow: 0 28px 80px rgba(10,12,14,.28); color: white; }
.coz-btn-dark { background: linear-gradient(180deg,#1b1b1b,#070707); color: white; box-shadow: 0 16px 32px rgba(0,0,0,.2); }
.coz-btn-light { background: rgba(255,255,255,.68); border: 1px solid rgba(22,18,14,.1); color: #141414; }
.coz-badge { border-radius: 999px; border: 1px solid rgba(22,18,14,.08); background: rgba(255,255,255,.68); }
.coz-light-field { position:absolute; inset:0; pointer-events:none; background: radial-gradient(circle at 74% 18%, rgba(255,209,129,.24), transparent 26%); }
```

- [ ] **步骤 3：创建 Primitives**

`Primitives.tsx` 必须导出：

```tsx
export function CozButton(...)
export function CozBadge(...)
export function CozCard(...)
export function CozSection(...)
export function CozIconOrb(...)
export function CozStat(...)
```

每个 primitive 只负责样式，不携带页面文案。

- [ ] **步骤 4：创建 Shell**

`Shell.tsx` 必须导出：

```tsx
export function CozPageShell({ children }: { children: React.ReactNode })
export function CozHeader()
export function CozFooter()
```

Header 必须复刻截图：

- 顶部浮动白色玻璃条。
- 左侧 logo + Cozmio。
- 中间导航：智能体、案例、桌面节点、提交任务。
- 右侧：中 / EN、GitHub、黑色提交任务按钮。
- 当前页面导航项有黑色下划线或黑色 active 状态，截图 5 Request 页必须显示 active underline。

- [ ] **步骤 5：运行构建**

执行命令：`npm.cmd run build`

预期结果：构建通过。若旧 `progress` 页面仍有 lint warning，不处理；本任务文件不得新增 TypeScript error。

## 任务 2：生成第一阶段视觉资产（首页 + Request）

**涉及文件**：

- 创建目录：`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\lib\cozmio-assets.ts`
- 修改：`D:\C_Projects\Agent\cozmio\web\next.config.ts`

- [ ] **步骤 1：生成首页 Hero 主视觉（资产 A）**

使用资产 A 提示词生成图片，复制为：

`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\home-task-core-hero.png`

验证命令：

```powershell
Get-Item 'D:\C_Projects\Agent\cozmio\web\public\images\cozmio\home-task-core-hero.png' | Select-Object FullName,Length
```

预期结果：文件存在，大小大于 `500000` bytes。

- [ ] **步骤 2：生成 Request 页视觉（资产 E）**

使用资产 E 提示词生成图片，复制为：

`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\request-task-core.png`

预期结果：文件存在，大小大于 `500000` bytes。

- [ ] **步骤 3：生成 CTA 视觉（资产 F）**

使用资产 F 提示词生成：

`D:\C_Projects\Agent\cozmio\web\public\images\cozmio\cta-core.png`

预期结果：文件存在，大小大于 `300000` bytes。

- [ ] **步骤 4：创建资产路径文件**

`cozmio-assets.ts` 内容：

```ts
export const cozmioAssets = {
  homeHero: "/images/cozmio/home-task-core-hero.png",
  requestHero: "/images/cozmio/request-task-core.png",
  ctaCore: "/images/cozmio/cta-core.png",
} as const;
```

- [ ] **步骤 5：确认静态导出图片配置**

`next.config.ts` 必须包含：

```ts
images: {
  unoptimized: true,
}
```

> 资产 B（Desktop 面板）、资产 C（Agents Core）、资产 D（Cases 缩略图）在第二阶段才生成。

---

## 任务 3：首页按截图 1 复刻

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\HomePage.tsx`
- 使用：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Shell.tsx`
- 使用：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Primitives.tsx`

- [ ] **步骤 1：替换首页入口**

`page.tsx` 只保留：

```tsx
import { HomePage } from "@/components/cozmio/HomePage";

export default function Page() {
  return <HomePage />;
}
```

- [ ] **步骤 2：实现首页 Hero**

`HomePage.tsx` 顶部必须复刻截图 1：

- Header 下方一个完整 hero 作品区，不是左右两栏硬切。
- 左侧文字：
  - `带着你的 Agent,`
  - `帮别人把东西做出来。`
  - `Cozmio 是一个 Agent Build Network。`
  - `Desktop Node 将你的电脑变成本地节点，连接真实项目、记忆与执行端点。`
- 三个 trust pills：
  - `真实项目`
  - `越用越懂你`
  - `安全可控`
- 两个按钮：
  - 黑色：`提交任务`
  - 白色：`探索网络`
- 右侧直接使用 `home-task-core-hero.png`，图像宽度约占 hero 右侧 58%，必须延伸到背景里，不能放进普通卡片。

- [ ] **步骤 3：实现三个入口卡片**

Hero 下方三张大卡片：

1. `发现 Agent`
2. `连接本地节点`
3. `沉淀交付案例`

每张卡片必须是截图中的大玻璃卡，不用普通 shadcn card。图标为大号发光 orb，使用 lucide 图标：

- Search
- Monitor
- FolderCheck

- [ ] **步骤 4：实现首页 Desktop 区块**

布局按截图：

- 左侧：badge `DESKTOP NODE`，标题 `把你的电脑 变成本地节点。`，副标题 `越用，系统越理解你。`
- 左侧三条 bullet：
  - `本地执行，数据不出你手`
  - `接入长期记忆，能力越用越强`
  - `连接执行端与工具链，高效可靠`
- 按钮：`下载 Desktop App`、`了解更多`
- 右侧使用 `desktop-node-panel.png` 或 React 深色面板复刻；优先使用生成图以保持高端视觉。

- [ ] **步骤 5：实现首页 Cases 区块**

标题：`真实案例`

副标题：`来自 Cozmio 网络的真实交付`

三张横向案例卡：

- `多租户 SaaS 管理平台`
- `销售数据分析与预测`
- `自动化内容生成流水线`

每张卡右侧放对应 case 缩略图，底部显示 `已交付`。

- [ ] **步骤 6：实现首页底部 CTA 与 Footer**

CTA 文案：

- `有想法？交给 Cozmio，让 Agent 帮你实现。`
- `发布任务后，网络中的 Agent 会为你报价、协作、交付。`

右侧使用 `cta-core.png`。

Footer 按截图：左侧 logo，四列链接，订阅输入框，社交圆形按钮。

- [ ] **步骤 7：首页视觉验收**

启动 dev server 后截图：

```powershell
npm.cmd run dev -- --hostname 127.0.0.1 --port 3000
```

浏览器检查：

- `http://127.0.0.1:3000/`
- 桌面宽度 `1440x2200`
- 首屏视觉必须像截图 1：暖米色、右侧大 3D Task Core、左侧粗黑标题、Header 浮动玻璃条。
- 页面不能出现旧的普通 SaaS 网格感。

## 任务 4：Desktop 页面按截图 2 复刻

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\desktop\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\DesktopPage.tsx`

- [ ] **步骤 1：替换路由入口**

`desktop/page.tsx` 只渲染 `DesktopPage`。

- [ ] **步骤 2：实现 Desktop Hero**

左侧：

- badge：`DESKTOP NODE`
- H1：
  - `把你的电脑，`
  - `变成 Agent 网络里的`
  - `本地节点。`
- subtitle：`越用，系统越理解你。`
- body：复刻截图中 Desktop Node 说明。
- 按钮：`下载 Desktop App`、`申请内测`
- 三个 trust pills：`数据本地优先`、`端到端加密`、`你完全掌控`

右侧：

- 使用 `desktop-node-panel.png`，尺寸和截图一致，深色面板不放白卡内。

- [ ] **步骤 3：实现三张能力视觉卡**

卡片标题：

- `本地执行`
- `长期记忆`
- `连接执行端`

每张卡底部使用轻量 generated/ CSS 玻璃小插画；如果资产不足，用 `cta-core.png` 的不同裁切作为背景，不使用随机小图标。

- [ ] **步骤 4：实现流程区**

标题：`从理解到交付，完整闭环。`

四步横向：

1. `观察`
2. `沉淀`
3. `调度`
4. `交付`

每步圆形 orb + 编号 + 文案，横向虚线/箭头连接，背景必须为整块浅色玻璃 section。

- [ ] **步骤 5：实现执行端与数据规则区**

执行端卡片：

- Claude Code
- Codex
- Gemini
- Custom CLI
- 添加执行端

数据规则区：

- `本地优先`
- `端到端加密`
- `仅限可控`
- `透明可审计`
- `开源可验证`

底部 pills：

- `SOC 2 Ready`
- `ISO 27001 设计原则`
- `本地隐私优先`
- `最小权限原则`

- [ ] **步骤 6：实现 Desktop CTA 与 Footer**

CTA 文案：

`立即启用 Desktop Node，让 Cozmio 真正理解并协同你的工作。`

按钮：`下载 Desktop App`、`申请内测`

- [ ] **步骤 7：Desktop 视觉验收**

检查 `http://127.0.0.1:3000/desktop/`：

- Hero 右侧深色面板必须是页面视觉中心。
- 页面整体应接近截图 2 的分区密度。
- 不得出现普通白卡片模板堆叠感。

## 任务 5：Agents 页面按截图 3 复刻

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\agents\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\AgentsPage.tsx`

- [ ] **步骤 1：替换路由入口**

`agents/page.tsx` 只渲染 `AgentsPage`。

- [ ] **步骤 2：实现 Agents Hero**

左侧：

- badge：`AGENTS`
- H1：
  - `可以被发现、`
  - `被雇用、被连接的 Agent。`
- body：`Cozmio 汇聚全球能力型 Agent，按能力、状态、连接节点与交付记录清晰呈现，帮助你更快找到值得信赖的执行者。`

右侧：

- 使用 `agents-core.png`。

- [ ] **步骤 3：实现搜索与筛选**

搜索框 placeholder：

`搜索 Agent 名称、能力、标签或描述...`

筛选按钮：

- `全部`
- `代码`
- `研究`
- `设计`
- `自动化`
- `已连接 Desktop Node`

右侧排序 dropdown 显示：`综合排序`

- [ ] **步骤 4：实现指标条**

三列指标：

- `1,248 可用 Agent +24 本周新增`
- `356 在线 Desktop Node 实时在线`
- `8,672 已完成交付 +12% 本周增长`

- [ ] **步骤 5：实现 Agent 网格**

两行四列卡片，标题：

- Cozmio Desktop Agent
- Claude Code Executor
- Research Scout
- Design Relay
- Automation Runner
- Data Analyst Pro
- Doc Writer
- 更多 Agent

每张卡必须包含：图标 orb、认证点、描述、标签、在线状态、已连接、评分、案例数。

- [ ] **步骤 6：实现推荐 Agent 大卡**

标题：`Cozmio Desktop Agent`

左侧大黑图标，中间能力与标签，底部指标：

- `在线`
- `已连接`
- `4.9 评分`
- `128 个案例`
- `98% 成功率`

右侧小面板：

- 已连接节点：`My Desktop Node`
- 近期交付三条
- 黑色按钮：`雇用此 Agent`

- [ ] **步骤 7：实现 Builder / 节点提供者**

三张人物卡：

- XiaoChen
- Lin Studio
- AutoCore

底部 CTA：

`找到匹配的 Agent，或提交你的任务。`

按钮：`浏览所有 Agent`、`提交任务`

- [ ] **步骤 8：Agents 视觉验收**

检查 `http://127.0.0.1:3000/agents/`：

- 顶部 hero 和截图 3 对齐。
- 搜索、筛选、指标条、Agent 网格、推荐卡、Builder 区齐全。
- 桌面宽度不溢出，卡片间距接近截图。

## 任务 6：Cases 页面按截图 4 复刻

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\cases\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\CasesPage.tsx`

- [ ] **步骤 1：替换路由入口**

`cases/page.tsx` 只渲染 `CasesPage`。

- [ ] **步骤 2：实现 Cases Hero**

左侧：

- badge：`CASES`
- H1：
  - `真实交付，`
  - `不只是演示。`
- body：复刻截图说明。
- stats：`180+ 真实案例`、`23+ 行业覆盖`、`98% 客户满意度`

右侧：

- 精选案例大卡：左图右文。
- 标题：`多租户 SaaS 管理平台`
- Agent pills：Gemini、Codex、Postgres
- 状态：`已交付`
- 按钮：`查看完整案例`

- [ ] **步骤 3：实现分类筛选**

筛选项：

- `全部`
- `Web 应用`
- `数据分析`
- `自动化`
- `研究`
- `设计`

Active 黑色按钮必须和截图一致。

- [ ] **步骤 4：实现 6 个案例网格**

案例：

- `销售数据分析与预测`
- `自动化内容生成流水线`
- `品牌官网与设计系统`
- `开发者 API 平台`
- `行业研究报告生成`
- `客户支持自动化助手`

每张卡必须使用对应生成缩略图，底部三项：天数、Agents 数量、满意度。

- [ ] **步骤 5：实现交付轨迹区**

标题：`交付轨迹`

四步：

1. Task
2. Agents
3. Delivery
4. Memory

每步带 icon orb、子 bullet、箭头连接。

- [ ] **步骤 6：实现 testimonial 与 CTA**

testimonial 文案：

`Cozmio 不只是帮我们写代码，更像是一个真正的数字团队。`

CTA：

`有项目想落地？让 Agent 帮你从想法到交付。`

按钮：`提交你的项目`、`浏览 Agent`

- [ ] **步骤 7：Cases 视觉验收**

检查 `http://127.0.0.1:3000/cases/`：

- Hero 左右比例接近截图 4。
- 6 个案例卡片缩略图比例一致。
- 交付轨迹不是普通 timeline，必须是整块浅色玻璃流程。

## 任务 7：Request 页面按截图 5 复刻

**涉及文件**：

- 修改：`D:\C_Projects\Agent\cozmio\web\src\app\request\page.tsx`
- 创建：`D:\C_Projects\Agent\cozmio\web\src\components\cozmio\RequestPage.tsx`

- [ ] **步骤 1：替换路由入口**

`request/page.tsx` 只渲染 `RequestPage`。

- [ ] **步骤 2：实现 Request Hero**

左侧：

- badge：`REQUEST`
- H1：
  - `告诉 Cozmio,`
  - `你想让 Agent 帮你做什么。`
- body：`提交你的任务，网络将为你匹配最适配的 Builder 与 Agent。当需要本地能力时，Desktop Node 也会加入协作。`
- 三个 trust pills：
  - `智能匹配最佳资源`
  - `端到端安全可控`
  - `全程透明可追踪`

右侧：

- 使用 `request-task-core.png`。

- [ ] **步骤 3：实现创建请求表单**

表单大卡标题：`创建请求`

字段按截图：

1. `你想做什么 *` textarea，placeholder `请描述你的任务目标、期望的成果、背景信息与关键要求...`，右下角 `0/1000`
2. `项目类型 *` select，placeholder `请选择项目类型`
3. `预算范围 *` select，placeholder `请选择预算范围`
4. `需要的 Agent 类型 *` select，placeholder `选择或搜索所需 Agent 类型（可多选）`
5. `交付时间 *` select，placeholder `请选择期望的交付时间`
6. `是否允许公开为案例` toggle，右侧显示 `允许`
7. `联系方式 *` input，placeholder `邮箱 / 手机号 / 其他联系方式`

底部按钮：`提交请求`

**表单必须能真实提交**：由于使用 Next static export，不做后端 API。选择以下任一方式：
1. Tally / Typeform / Google Form 嵌入（推荐，最快上线）
2. Formspree / Formspark 等外部 form endpoint
3. `mailto:` 作为最低兜底

表单字段至少收集：任务描述、项目类型、预算范围、交付时间、联系方式、是否允许公开为案例。

旁边隐私提示：`你的信息将被严格保密，仅用于任务匹配与协作`

- [ ] **步骤 4：实现提交流程侧栏**

右侧卡片标题：`提交流程`

四步：

1. `提交需求`
2. `网络评估`
3. `Builder / Agent 接入`
4. `交付与沉淀`

底部三条承诺：

- `预计首次响应 ≈ 30 分钟内`
- `安全与隐私 端到端加密，严格保密`
- `全程可追踪 任务进度实时可见`

- [ ] **步骤 5：实现适合做什么与近期请求案例**

左卡：`适合做什么`

标签：

- 软件开发
- 数据分析
- 自动化脚本
- 设计创意
- 研究报告
- 内容生成
- 产品原型
- 更多场景...

右卡：`近期请求案例`

三张小卡：

- `电商数据分析看板`
- `企业内部知识库搭建`
- `自动化运营脚本开发`

- [ ] **步骤 6：实现快速示例与底部 CTA**

快速示例标题：

`不知道怎么描述？试试这些快速示例`

三张示例：

- `开发一个数据看板`
- `搭建 AI 客服机器人`
- `自动化报表生成`

CTA：

`准备好了吗？现在就提交你的请求。`

按钮：`立即提交请求`

- [ ] **步骤 7：Request 视觉验收**

检查 `http://127.0.0.1:3000/request/`：

- Header 当前项必须在提交任务下有黑色 active underline。
- Hero 右侧视觉必须接近截图 5，不是当前普通 Agent Character 图。
- 表单字段顺序、编号、大小、间距必须接近截图。

## 双语策略：延期

第一版只做中文。

Header 保留 "中 / EN" 视觉，但 EN 暂不切换，或点击后显示 Coming soon。

原因：

- 双语会拉宽文案；
- 增加数据结构复杂度；
- 增加布局 bug；
- 现在不会帮你更快赚钱。

英文版在首页和 Request 验证后再做。

## 任务 9：响应式与稳定性

**涉及文件**：

- 修改：所有新页面和 Cozmio 组件。

- [ ] **步骤 1：桌面端优先验收**

主验收宽度：

- `1440px`
- `1536px`
- `1920px`

截图宽度 `1440px` 时，页面应接近用户设计稿。

- [ ] **步骤 2：移动端基础可用**

移动端不要求与截图一致，但必须：

- 无水平滚动。
- Header 可横向滚动或折行不遮挡。
- Hero 图片不压扁。
- 表单字段不溢出。

- [ ] **步骤 3：静态导出稳定**

执行：

```powershell
npm.cmd run build
```

预期结果：退出码为 `0`，新页面全部出现在 Route 输出中：

- `/`
- `/agents`
- `/cases`
- `/desktop`
- `/request`

- [ ] **步骤 4：Lint 稳定**

构建标准：

- `npm.cmd run build` 必须通过，退出码为 `0`。
- 如果项目已有稳定 lint 脚本，则运行 `npm.cmd run lint`，退出码为 `0`。
- 如果 lint 脚本不存在或旧文件已有 warning，**不允许为本次重构大改 lint 配置**。
- 本次新增文件不得引入 TypeScript error。

> 不要为了 lint 去改配置。Codex 很可能会把事情搞复杂。

## 任务 10：截图级视觉验收

**涉及文件**：

- 创建目录：`D:\C_Projects\Agent\cozmio\web\.visual-checks\`

- [ ] **步骤 1：启动 dev server**

执行：

```powershell
npm.cmd run dev -- --hostname 127.0.0.1 --port 3000
```

预期结果：输出 `http://127.0.0.1:3000`。

- [ ] **步骤 2：采集桌面截图**

用 Chrome DevTools 或 Playwright 对以下页面截全页图：

- `/`
- `/desktop`
- `/agents`
- `/cases`
- `/request`

截图保存到：

```text
D:\C_Projects\Agent\cozmio\web\.visual-checks\home.png
D:\C_Projects\Agent\cozmio\web\.visual-checks\desktop.png
D:\C_Projects\Agent\cozmio\web\.visual-checks\agents.png
D:\C_Projects\Agent\cozmio\web\.visual-checks\cases.png
D:\C_Projects\Agent\cozmio\web\.visual-checks\request.png
```

- [ ] **步骤 3：人工对照用户设计稿**

逐页检查：

- Header 高度、圆角、阴影是否接近截图。
- Hero 是否是一张完整作品，不是左右拼接。
- 主视觉大小和位置是否接近截图。
- Section 是否延续暖米色、玻璃、暖金光线语言。
- 深色 Desktop 面板是否足够高级。
- 卡片是否有统一边框、阴影、圆角。
- 中文模式是否不混英文普通 UI 文案。
- 页面是否仍保留 Agent Build Network + Desktop Node + 越用越懂你的叙事。

- [ ] **步骤 4：记录视觉偏差并修复**

如果任一页面出现以下问题，必须修正后重新截图：

- 像普通 SaaS 模板。
- 主视觉像随机 AI 图，和版式脱节。
- 左文右图割裂。
- Section 掉回普通白卡片堆叠。
- 图标过多且无意义。
- 中文页面出现大量英文 UI 文案。
- 桌面端宽度有水平滚动。

## 完成标准

完成后必须满足：

- 5 个核心页面与 5 张设计稿的结构一一对应。
- 首页 hero 有完整作品感，右侧主视觉可直接使用生成图，但必须和左侧文案融合。
- Desktop 页拥有截图级深色节点面板视觉。
- Agents 页有搜索、筛选、指标、Agent 网格、推荐 Agent、Builder 区。
- Cases 页有精选案例、分类、6 案例网格、交付轨迹、testimonial、CTA。
- Request 页有编号表单、提交流程侧栏、适合做什么、近期请求、快速示例、CTA。
- Header / Footer / Button / Card / Badge / Section 共享同一套设计系统。
- 不引入 WebGL、不引入 react-three-fiber、不引入 drei、不引入不稳定依赖。
- `npm.cmd run build` 通过。
- `npm.cmd run lint` 退出码为 0，且本次新增文件没有 warning。
- 生成的截图保存在 `.visual-checks/`，用于和用户设计稿对照。
