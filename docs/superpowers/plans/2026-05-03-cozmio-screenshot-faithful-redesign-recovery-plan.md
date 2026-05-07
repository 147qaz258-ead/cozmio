# Cozmio 官网截图级复刻修复实施方案

> **智能执行体须知**：必需子技能——使用 `superpowers:subagent-driven-development`（推荐）或 `superpowers:executing-plans` 逐任务落地本方案。步骤使用复选框（`- [ ]`）语法进行跟踪。

**目标**：以用户设计稿为唯一视觉基准，修复当前 `web/` 官网“左文右图、图片呆站、普通卡片堆叠”的问题，把首页、Agents、Desktop、Cases、Request 改成截图级的“视觉底层 + 内容上层 + 产品发布页作品感”，并对提交任务、Desktop 下载等需要真实能力的入口接入真实后端。

**架构思路**：保留 Next.js 16 静态导出和 Tailwind 4，但重建 Cozmio 页面组合方式：不再用简单 `grid lg:grid-cols[...]` 作为 hero 主结构，而是引入 `StageHero`、`LayeredVisual`、`OverlayPanel` 等叠层组件。图片资产作为页面空间底层或右侧舞台背景，文字、搜索、筛选、表单和状态卡作为上层叠加元素。真实交互通过独立 Cloudflare Worker 接入，避免破坏静态导出。

**技术栈**：Next.js 16.2.2、React 19.2.4、TypeScript、Tailwind CSS 4、lucide-react、Cloudflare Pages、Cloudflare Worker、GitHub Releases、Playwright/Chrome screenshot verification。

---

## 1. 产品类型

传统软件实现型 + 视觉运行体验型。

核心风险不是组件是否存在，而是视觉层级是否和设计稿一致、真实页面是否仍像模板、资产是否融入版式、桌面端和移动端是否无溢出、表单和下载入口是否真实可用。

## 2. 当前真相

- `/agents/` 本地返回 200，用户截图就是当前真实页面。
- 图片资产已存在于 `web/public/images/cozmio/`，且大小足够，不是缺图：
  - `home-task-core-hero.png`
  - `agents-core.png`
  - `desktop-node-panel.png`
  - `request-task-core.png`
  - `cta-core.png`
  - case thumbnail images
- 当前 `AgentsPage.tsx`、`DesktopPage.tsx`、`CasesPage.tsx`、`RequestPage.tsx` 仍以 `grid lg:grid-cols[...]` 组织 hero，文字在左，图片在右。
- 当前 `HeroVisual` 只是普通 `Image object-contain`，没有承担“背景舞台 / 底层空间”的职责。
- 当前 Request 提交使用 `mailto:`，不是用户要求的真实后端。
- 当前 Desktop 下载按钮链接到 `/desktop` 自己，不是真实下载。
- `update-server` 已有 Cloudflare Worker 和 GitHub Release 查询逻辑，但只有 `/updates/check`，缺少官网下载 endpoint 和任务请求收集 endpoint。
- `web` 是 Next static export，不能依赖 Next API routes。

## 3. 设计稿差异诊断

### 3.1 最大偏差：画面结构错了

当前页面是：

```text
左侧文字块 | 右侧图片块
下方搜索框
下方筛选器
下方统计卡
```

设计稿要的是：

```text
整屏暖米色产品舞台
  图片/光线/3D 核心作为底层空间
  标题、搜索、筛选、状态卡、表单作为上层浮层
  各层互相压叠，形成一张完整 PPT 式作品页
```

### 3.2 图片使用方式错了

当前图片边界非常清楚，像一个独立插图“呆呆立在那里”。设计稿里的图片应该：

- 融入暖米色背景，不显露硬矩形边缘。
- 允许被文字区、搜索区、卡片区部分压住或穿插。
- 有光晕、雾化、遮罩、渐隐和景深。
- 不被普通 card 包起来。

### 3.3 内容密度错了

当前 hero 下方很快进入表格式控件，缺少设计稿的“舞台感”和视觉留白。应先让首屏建立高端产品印象，再让搜索/筛选/表单进入上层交互。

### 3.4 真实交互不足

- Request：必须提交到真实服务，至少落到 Worker 后端并返回提交成功。
- Desktop 下载：必须指向真实最新版 installer 或明确的 release 下载 endpoint。
- GitHub：必须用真实 repo URL。
- Agent 筛选/搜索：不必接数据库，但至少要有前端真实过滤，不能只是静态摆设。

## 4. 文件结构

### 修改

- `D:\C_Projects\Agent\cozmio\web\src\app\globals.css`
  - 增加叠层舞台、图片渐隐、上层浮层、截图验收辅助样式。
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Primitives.tsx`
  - 增加 `CozLayerCard`、`CozFloatingPill`、`CozSectionTitle` 等更适合叠层布局的基础件。
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\Shell.tsx`
  - 修正 GitHub 链接；Header 适配设计稿 active 状态。
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\VisualPanels.tsx`
  - 替换 `HeroVisual` 语义，新增 `StageHeroVisual`、`DesktopDownloadPanel`、`RequestFlowPanel`。
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\HomePage.tsx`
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\AgentsPage.tsx`
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\DesktopPage.tsx`
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\CasesPage.tsx`
- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\RequestPage.tsx`
- `D:\C_Projects\Agent\cozmio\web\src\lib\cozmio-assets.ts`
- `D:\C_Projects\Agent\cozmio\web\src\lib\site-config.ts`
- `D:\C_Projects\Agent\cozmio\update-server\src\index.js`
- `D:\C_Projects\Agent\cozmio\update-server\README.md`

### 创建

- `D:\C_Projects\Agent\cozmio\web\src\components\cozmio\StageHero.tsx`
  - 全站叠层 hero 布局核心。
- `D:\C_Projects\Agent\cozmio\web\src\lib\cozmio-data.ts`
  - Agent、Case、Request option、Desktop download copy 集中数据。
- `D:\C_Projects\Agent\cozmio\web\src\lib\request-submit.ts`
  - 静态站前端提交到 Worker 的客户端封装。
- `D:\C_Projects\Agent\cozmio\web\.visual-checks\`
  - 保存对照截图。

## 5. 实施任务

### 任务 1：建立叠层舞台系统

**涉及文件**：

- 创建：`web/src/components/cozmio/StageHero.tsx`
- 修改：`web/src/app/globals.css`
- 修改：`web/src/components/cozmio/Primitives.tsx`
- 修改：`web/src/components/cozmio/VisualPanels.tsx`

- [ ] **步骤 1：创建 `StageHero.tsx`**

实现一个不使用左右硬切 grid 的 hero 容器，支持视觉在底层、文字和控件在上层：

```tsx
import Image from "next/image";
import type { ReactNode } from "react";

type StageHeroProps = {
  eyebrow: ReactNode;
  title: ReactNode;
  body: ReactNode;
  visualSrc: string;
  visualAlt: string;
  actions?: ReactNode;
  pills?: ReactNode;
  overlay?: ReactNode;
  align?: "left" | "center";
};

export function StageHero({
  eyebrow,
  title,
  body,
  visualSrc,
  visualAlt,
  actions,
  pills,
  overlay,
  align = "left",
}: StageHeroProps) {
  return (
    <section className="coz-shell">
      <div className="coz-stage-hero">
        <div className="coz-stage-light" />
        <Image
          src={visualSrc}
          alt={visualAlt}
          width={1280}
          height={820}
          priority
          className="coz-stage-visual"
        />
        <div className={align === "center" ? "coz-stage-copy center" : "coz-stage-copy"}>
          {eyebrow}
          <h1>{title}</h1>
          <p>{body}</p>
          {pills ? <div className="coz-stage-pills">{pills}</div> : null}
          {actions ? <div className="coz-stage-actions">{actions}</div> : null}
        </div>
        {overlay ? <div className="coz-stage-overlay">{overlay}</div> : null}
      </div>
    </section>
  );
}
```

- [ ] **步骤 2：添加舞台 CSS**

在 `globals.css` 的 `@layer components` 增加：

```css
.coz-stage-hero {
  position: relative;
  min-height: 720px;
  overflow: hidden;
  border-radius: 34px;
  isolation: isolate;
  background:
    radial-gradient(circle at 76% 18%, rgba(255, 214, 146, .28), transparent 30%),
    radial-gradient(circle at 22% 28%, rgba(255, 255, 255, .82), transparent 34%),
    linear-gradient(135deg, rgba(255,255,255,.34), rgba(245,236,222,.18));
}

.coz-stage-light {
  position: absolute;
  inset: -12% -8%;
  z-index: 0;
  background:
    linear-gradient(115deg, rgba(255,255,255,.78) 0%, transparent 38%),
    radial-gradient(circle at 70% 40%, rgba(245,181,68,.24), transparent 34%);
  pointer-events: none;
}

.coz-stage-visual {
  position: absolute;
  right: -6%;
  top: 4%;
  z-index: 1;
  width: min(68%, 940px);
  height: auto;
  object-fit: contain;
  filter: drop-shadow(0 48px 90px rgba(90,63,28,.16));
  mix-blend-mode: multiply;
  mask-image: radial-gradient(circle at 58% 48%, #000 0 58%, transparent 78%);
}

.coz-stage-copy {
  position: relative;
  z-index: 3;
  max-width: 680px;
  padding: 104px 0 80px 0;
}

.coz-stage-copy h1 {
  font-size: clamp(52px, 6vw, 92px);
  line-height: .98;
  font-weight: 950;
  letter-spacing: 0;
  color: #090909;
}

.coz-stage-copy p {
  margin-top: 30px;
  max-width: 640px;
  font-size: 20px;
  line-height: 1.9;
  color: #625b54;
}

.coz-stage-pills,
.coz-stage-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 14px;
  margin-top: 30px;
}

.coz-stage-overlay {
  position: relative;
  z-index: 4;
}
```

- [ ] **步骤 3：移动端样式**

追加：

```css
@media (max-width: 900px) {
  .coz-stage-hero {
    min-height: auto;
    padding-bottom: 340px;
  }

  .coz-stage-copy {
    padding: 70px 0 36px 0;
  }

  .coz-stage-copy h1 {
    font-size: clamp(42px, 12vw, 64px);
  }

  .coz-stage-visual {
    top: auto;
    right: 50%;
    bottom: 0;
    width: 112%;
    transform: translateX(50%);
  }
}
```

- [ ] **步骤 4：运行构建**

执行命令：

```powershell
npm.cmd run build
```

预期结果：退出码 `0`。

### 任务 2：首页按叠层设计稿重做

**涉及文件**：

- 修改：`web/src/components/cozmio/HomePage.tsx`
- 使用：`web/src/components/cozmio/StageHero.tsx`

- [ ] **步骤 1：替换首页 hero 结构**

删除首页顶部 `grid min-h-[680px] ... lg:grid-cols[...]`，改为 `StageHero`：

```tsx
<StageHero
  eyebrow={<CozBadge>COZMIO</CozBadge>}
  title={<>带着你的 Agent,<br />帮别人把东西做出来。</>}
  body={<>Cozmio 是一个 Agent Build Network。<br />Desktop Node 将你的电脑变成本地节点，连接真实项目、记忆与执行端点。</>}
  visualSrc={cozmioAssets.homeHero}
  visualAlt="Cozmio Task Core Agent 网络"
  pills={...}
  actions={...}
  overlay={<HomeHeroEntrances /> }
/>
```

- [ ] **步骤 2：把三个入口卡片压到 hero 底部**

创建 `HomeHeroEntrances`，三张卡不再是普通 section，而是 hero 内部底部浮层：

```tsx
function HomeHeroEntrances() {
  return (
    <div className="coz-hero-entrances">
      {/* 发现 Agent / 连接本地节点 / 沉淀交付案例 */}
    </div>
  );
}
```

CSS：

```css
.coz-hero-entrances {
  position: absolute;
  left: 36px;
  right: 36px;
  bottom: 32px;
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 18px;
}
```

- [ ] **步骤 3：首页 Desktop 区改为视觉主导**

Desktop 区块不能左文右面板割裂。改成一整块深色面板作为背景舞台：

```text
左上：DESKTOP NODE + 标题
右侧/底层：desktop-node-panel.png 大图
左下：三条 bullet 和下载按钮
```

`desktop-node-panel.png` 不再被普通 grid 推到右栏，而是 absolute 背景图，文字叠在上层。

- [ ] **步骤 4：首页 CTA 改为图文融合**

`cta-core.png` 使用 absolute 背景，CTA 文案和按钮叠在左上，不再是 `flex justify-between` 的左右拼接。

- [ ] **步骤 5：首页截图验收**

启动：

```powershell
npm.cmd run dev -- --hostname 127.0.0.1 --port 3000
```

截图：

```powershell
node .\scripts\capture-page.mjs http://127.0.0.1:3000/ .visual-checks/home.png
```

预期：

- 首屏不是左文右图。
- 右侧图片没有硬矩形边界。
- 三张入口卡在 hero 底部形成上层浮层。
- 图片、光线、文字像一张完整产品发布页。

### 任务 3：Agents 页面重做为设计稿式叠层目录页

**涉及文件**：

- 修改：`web/src/components/cozmio/AgentsPage.tsx`
- 创建或修改：`web/src/lib/cozmio-data.ts`

- [ ] **步骤 1：Hero 改用 `StageHero`**

删除 `lg:grid-cols` hero，改成 `StageHero`，把 `agents-core.png` 放底层。

- [ ] **步骤 2：搜索和筛选上移为 hero 浮层**

搜索框、筛选按钮、排序按钮不再是独立 section，而是 `StageHero.overlay` 底部浮层。它们应压住图片底部区域，形成设计稿的 PPT 层级。

- [ ] **步骤 3：实现前端真实筛选**

`agents` 数据移入 `cozmio-data.ts`：

```ts
export type AgentCategory = "全部" | "代码" | "研究" | "设计" | "自动化" | "已连接 Desktop Node";

export const agentItems = [
  {
    name: "Cozmio Desktop Agent",
    categories: ["代码", "自动化", "已连接 Desktop Node"],
    description: "...",
    connected: true,
    rating: "4.9",
    cases: 128,
  },
] as const;
```

`AgentsPage` 使用 `useState` 对 search query 和 category 做前端过滤。

- [ ] **步骤 4：Agent 网格视觉降噪**

卡片仍可保留，但必须低于 hero 视觉层级，不抢首屏。减少随机 icon 堆砌，统一使用 orb + 状态 + 指标。

- [ ] **步骤 5：Agents 截图验收**

保存：

```text
web/.visual-checks/agents.png
```

预期：

- 对比用户提供的当前截图，新的首屏不再是“图片一坨在右、文字一坨在左”。
- 搜索筛选与 hero 有叠层关系。
- 首屏视觉接近设计稿，而不是普通 SaaS agent directory。

### 任务 4：Request 页面接入真实后端并改为叠层请求页

**涉及文件**：

- 修改：`web/src/components/cozmio/RequestPage.tsx`
- 创建：`web/src/lib/request-submit.ts`
- 修改：`update-server/src/index.js`
- 修改：`update-server/README.md`

- [ ] **步骤 1：Request Hero 改用 `StageHero`**

`request-task-core.png` 作为底层空间图，标题、trust pills 叠在上层。

- [ ] **步骤 2：表单区保持设计稿编号结构**

表单大卡可以保留，但必须向上压入 hero 下沿，让它像页面上层组件，而不是另一个普通 section。

- [ ] **步骤 3：Worker 新增 `POST /requests`**

在 `update-server/src/index.js` 增加路由：

```js
if (url.pathname === "/requests" && request.method === "POST") {
  const payload = await request.json();
  const required = ["task", "projectType", "budget", "timeline", "contact"];
  for (const key of required) {
    if (!payload[key] || String(payload[key]).trim().length === 0) {
      return jsonResponse({ ok: false, error: `${key} required` }, 400);
    }
  }

  console.log("Cozmio request", JSON.stringify({
    task: payload.task,
    projectType: payload.projectType,
    budget: payload.budget,
    agentType: payload.agentType || "",
    timeline: payload.timeline,
    publicCase: Boolean(payload.publicCase),
    contact: payload.contact,
    createdAt: new Date().toISOString(),
  }));

  return jsonResponse({ ok: true, message: "request received" });
}
```

第一版先写 Worker log。后续若需要持久化，再接 Cloudflare D1、KV、Resend 或邮件 webhook。

- [ ] **步骤 4：Worker 增加 CORS**

`jsonResponse` 支持：

```js
"Access-Control-Allow-Origin": "https://cozmio.net",
"Access-Control-Allow-Methods": "GET,POST,OPTIONS",
"Access-Control-Allow-Headers": "Content-Type"
```

并处理 `OPTIONS` 预检。

- [ ] **步骤 5：前端替换 `mailto:`**

`request-submit.ts`：

```ts
const REQUEST_ENDPOINT =
  process.env.NEXT_PUBLIC_COZMIO_REQUEST_ENDPOINT ??
  "https://cozmio-updates.workers.dev/requests";

export async function submitCozmioRequest(payload: Record<string, unknown>) {
  const response = await fetch(REQUEST_ENDPOINT, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });

  const result = await response.json().catch(() => ({}));
  if (!response.ok || result.ok === false) {
    throw new Error(result.error || "提交失败，请稍后再试");
  }
  return result;
}
```

`RequestPage` 提交后显示成功状态，不跳 mail client。

- [ ] **步骤 6：Request 验收**

执行 Worker 本地测试：

```powershell
cd D:\C_Projects\Agent\cozmio\update-server
npx wrangler dev
```

提交页面表单，预期：

- Network 里 `POST /requests` 返回 200。
- 页面出现成功提示。
- 缺少必填字段时返回错误并显示到表单上。

### 任务 5：Desktop 下载接入真实 Release

**涉及文件**：

- 修改：`update-server/src/index.js`
- 修改：`web/src/components/cozmio/DesktopPage.tsx`
- 修改：`web/src/components/cozmio/HomePage.tsx`
- 修改：`web/src/components/cozmio/VisualPanels.tsx`
- 修改：`web/src/lib/site-config.ts`

- [ ] **步骤 1：Worker 新增 `GET /downloads/windows`**

复用 `fetchLatestRelease(env)`，查找 `.msi` asset，302 跳转到 `browser_download_url`：

```js
if (url.pathname === "/downloads/windows") {
  const release = await fetchLatestRelease(env);
  if (!release) {
    return jsonResponse({ error: "No release found" }, 404);
  }
  const msiAsset = findAssetBySuffix(release.assets, ".msi");
  if (!msiAsset) {
    return jsonResponse({ error: "No Windows installer found" }, 404);
  }
  return Response.redirect(msiAsset.browser_download_url, 302);
}
```

- [ ] **步骤 2：集中配置下载链接**

`site-config.ts` 增加：

```ts
downloads: {
  windows: "https://cozmio-updates.workers.dev/downloads/windows",
}
```

- [ ] **步骤 3：替换所有 Desktop 下载按钮**

把以下按钮的 `href="/desktop"` 改为 `siteConfig.downloads.windows`：

- 首页 Desktop 区 `下载 Desktop App`
- Desktop 页 hero `下载 Desktop App`
- Desktop 页 CTA `下载 Desktop App`

使用普通 `<a>` 外链下载，不用 Next `Link` 内链。

- [ ] **步骤 4：下载验收**

执行：

```powershell
Invoke-WebRequest -Uri "https://cozmio-updates.workers.dev/downloads/windows" -MaximumRedirection 0
```

预期：

- 如果已有 GitHub Release MSI：返回 `302`。
- 如果没有 release：返回明确 JSON 错误，页面按钮旁显示“内测下载即将开放”兜底文案。

### 任务 6：Cases 和 Desktop 页面二阶段复刻

**涉及文件**：

- 修改：`web/src/components/cozmio/DesktopPage.tsx`
- 修改：`web/src/components/cozmio/CasesPage.tsx`

- [ ] **步骤 1：Desktop hero 改为深色面板主视觉**

用 `StageHero` 或专用 `DesktopStage`，让 `desktop-node-panel.png` 成为页面右侧/底层视觉中心，不放白卡内。

- [ ] **步骤 2：Desktop 后续 section 减少普通卡片堆叠**

三张能力卡、流程区、执行端、数据规则区必须形成整块 section band，不能像模板卡片网格。

- [ ] **步骤 3：Cases hero 改为精选案例叠层**

精选案例大卡与背景图层融合，左侧标题、右侧案例图卡不是普通 `grid`。

- [ ] **步骤 4：Cases 网格保留但统一视觉**

6 张案例缩略图保持一致比例，减少文字密度，突出图片和已交付状态。

### 任务 7：图片资产补强

**涉及文件**：

- 修改或替换：`web/public/images/cozmio/*.png`

- [ ] **步骤 1：检查每张图片是否有硬矩形边界**

对以下图片在页面中截图检查：

- `home-task-core-hero.png`
- `agents-core.png`
- `request-task-core.png`
- `desktop-node-panel.png`
- `cta-core.png`

若图片背景和页面底色不融合，重新生成透明感更强、左侧留白更多、边缘更可渐隐的版本。

- [ ] **步骤 2：重新生成 Agents 图**

当前用户截图中 `agents-core.png` 最明显“呆站”。重新生成时要求：

```text
Create a wide premium website background visual for a Cozmio agents directory hero. Warm ivory space, transparent 3D agent core on the right, broad empty light field on the left for Chinese headline overlay, soft champagne light trails crossing behind the text area, no hard rectangular panel edge, no readable text, no logo, no robot, no cyberpunk. The visual must work as a page background layer, not as a standalone illustration card.
```

- [ ] **步骤 3：重新生成首页图**

要求同上，重点是左侧可叠文字、右侧核心不被硬裁切。

- [ ] **步骤 4：重新生成 Request 图**

要求右侧 task core 和底部表单能形成上下层关系。

### 任务 8：截图级验收与回归

**涉及文件**：

- 创建或修改：`web/scripts/capture-page.mjs`
- 创建目录：`web/.visual-checks/`

- [ ] **步骤 1：创建截图脚本**

`capture-page.mjs` 使用 Playwright 打开 URL，设置 1440 宽，保存 full page screenshot。

- [ ] **步骤 2：采集页面**

执行：

```powershell
node .\scripts\capture-page.mjs http://127.0.0.1:3000/ .visual-checks/home.png
node .\scripts\capture-page.mjs http://127.0.0.1:3000/agents/ .visual-checks/agents.png
node .\scripts\capture-page.mjs http://127.0.0.1:3000/desktop/ .visual-checks/desktop.png
node .\scripts\capture-page.mjs http://127.0.0.1:3000/cases/ .visual-checks/cases.png
node .\scripts\capture-page.mjs http://127.0.0.1:3000/request/ .visual-checks/request.png
```

- [ ] **步骤 3：人工对照设计稿**

逐页检查：

- 是否仍是左右两栏。
- 图片是否仍像单独站着。
- 文字是否压到视觉上层。
- 搜索/筛选/表单是否和 hero 有叠层关系。
- 是否出现普通 SaaS 模板感。
- Header、Footer、按钮、卡片是否统一。

- [ ] **步骤 4：构建与 lint**

```powershell
npm.cmd run build
npm.cmd run lint
```

预期：

- `npm.cmd run build` 退出码 `0`。
- 本次新增或修改文件没有 TypeScript error。
- 若旧文件已有 warning，不修改 lint 配置来掩盖。

## 6. 阶段门

### 第一阶段完成标准

- [ ] 首页 hero 改成叠层舞台，不再左右硬切。
- [ ] Agents hero 改成叠层舞台，搜索和筛选作为上层浮层。
- [ ] Request hero 改成叠层舞台，表单压入视觉结构。
- [ ] `POST /requests` 可真实提交。
- [ ] Desktop 下载按钮不再指向 `/desktop` 自己。
- [ ] `.visual-checks/home.png`、`.visual-checks/agents.png`、`.visual-checks/request.png` 已生成。
- [ ] `npm.cmd run build` 通过。

### 第二阶段完成标准

- [ ] Desktop 页面深色节点面板成为视觉中心。
- [ ] Cases 页面不再是普通精选卡 + 网格模板。
- [ ] 5 个核心页面截图均已生成。
- [ ] 所有设计稿差异已记录并修复。

## 7. 执行优先级

1. 先修首页、Agents、Request 的叠层结构，因为这是用户肉眼最强烈感知的问题。
2. 再接真实 Request 后端和 Desktop 下载，因为这是转化路径。
3. 最后修 Desktop/Cases 二级页面和图片细节。

不要先做小图标、微动画或双语。当前最重要的是：页面第一眼必须像设计稿，而不是像一个有 3D 插图的普通 SaaS 模板。
