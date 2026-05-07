# Cozmio V1 前端功能修复与补全设计方案

**版本：** 1.0
**日期：** 2026-05-06
**状态：** 待审核
**验证基准：** `https://cozmio.net` Playwright 端到端验证（2026-05-06）

---

## 一、问题总览（部署验证后）

### A. 阻断性问题（完全不可用）

| # | 页面/功能 | 问题描述 | 严重度 |
|---|---------|---------|--------|
| A1 | `/request` | 页面 404，路由不存在 | 🔴 阻断 |
| A2 | `/use` Email 登录 | API 请求发往 `http://47.76.116.209`（HTTP），浏览器 block mixed content | 🔴 阻断 |
| A3 | Footer 订阅更新 | 输入邮箱后点击按钮无任何反应（无 API，无 submit 逻辑） | 🔴 阻断 |
| A4 | Footer 产品/资源/公司链接 | 文字是 `<p>` 纯文本，没有 `<a>` 标签，无法点击 | 🔴 阻断 |

### B. 功能性问题（可用但不完整）

| # | 页面/功能 | 问题描述 | 严重度 |
|---|---------|---------|--------|
| B1 | Cloudflare Pages RSC prefetch | 控制台大量 `Failed to load resource: 404` — `/agents/__next.agents.__PAGE__.txt` 等。这是 `output: "export"` 在 Cloudflare Pages 上的已知问题，不影响功能但污染控制台 | 🟡 功能正常 |
| B2 | `/agents` 智能体页面 | 页面显示大量假数据（1,248 个 Agent），搜索框输入后无过滤，雇用按钮指向 `/request`（404）。无真实 API | 🟡 假数据 |
| B3 | `/use` 页面 | 英文 "Welcome to Cozmio"，与整体中文风格不一致 | 🟡 UI 问题 |
| B4 | 下载 Desktop App 链接 | `http://47.76.116.209/api/downloads/latest?platform=windows` — HTTP 链接在 HTTPS 页面被 block | 🟡 已拦截 |
| B5 | Footer 社交图标 | GH/X/DC/◎ 图标显示，但没有 href 链接 | 🟡 无链接 |
| B6 | 落地页导航"提交任务"按钮 | 指向 `/request`（404） | 🟡 404 跳转 |

### C. 已正常工作的页面

| 页面 | 状态 | 说明 |
|------|------|------|
| `/` 落地页 | ✅ 可用 | 导航存在，内容完整 |
| `/agents/` | ✅ 可用（假数据） | 页面能访问，但数据是 hardcode mock |
| `/cases/` | ✅ 可用 | 页面能访问，有案例内容 |
| `/desktop/` | ✅ 可用 | 页面能访问，下载按钮被 block |
| `/blog/` | ✅ 可用 | 页面能访问，标题仍为 "Pulseclaw" 品牌 |
| `/about/` | ✅ 可用 | Contact 内容完整 |
| `/contact/` | ✅ 可用 | 社交链接完整（GitHub/X/公众号） |
| `/privacy/` | ✅ 可用 | 隐私政策页面存在 |
| `/terms/` | ✅ 可用 | 服务条款页面存在 |
| `/use/` | ✅ 可访问（不可用） | 页面存在但 API 不通 |
| `/admin/` | ✅ 可访问（不可用） | 管理后台存在但 API 不通 |
| `/admin/dashboard/` | ✅ 可访问（不可用） | 同上 |
| `/admin/applications/` | ✅ 可访问（不可用） | 同上 |

---

## 二、修复方案

### RP-1: Footer 链接修复

**文件**: `web/src/components/layout/Footer.tsx`

**问题**: Footer "产品"/"资源"/"公司" 下的文字是 `<p>` 元素，没有包裹 `<a>`，无法点击。

**修改为**: 将每个 `<p>` 改为 `<Link href="...">` 包裹的可点击链接。

```tsx
// 产品列 - 修改前
<p>智能体</p>
<p>桌面节点</p>
// 修改后
<Link href="/agents/" className="text-sm hover:underline">智能体</Link>
<Link href="/desktop/" className="text-sm hover:underline">桌面节点</Link>
```

**链接目标**:
- 产品：智能体 → `/agents/`，桌面节点 → `/desktop/`，提交任务 → `/request/`
- 资源：案例 → `/cases/`，文档 → `/docs/`（待创建），帮助中心 → `/help/`（待创建）
- 公司：关于我们 → `/about/`（已有），隐私政策 → `/privacy/`，服务条款 → `/terms/`

---

### RP-2: API 反向代理配置（解决 Mixed Content）

**问题**: 前端 HTTPS 页面无法请求 HTTP 后端 `http://47.76.116.209`。

**方案**: nginx 反向代理，让 `/api/*` 请求在 Cloudflare 边缘（或服务器端）转发到后端。

```nginx
server {
    listen 80;
    server_name cozmio.net www.cozmio.net;

    # API 反向代理
    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_ssl_server_name on;
    }

    # 静态文件
    location / {
        root /var/www/cozmio/out;
        try_files $uri $uri/ /index.html;
    }
}
```

**后端启动命令**（已在服务器设置，需验证）:
```bash
# 确保后端在 localhost:3000 运行
# nginx 反向代理 /api/* → localhost:3000
```

**前端 `.env.production`**:
```
NEXT_PUBLIC_API_BASE_URL=/api
```
（这样前端请求 `/api/xxx`，通过同源 nginx 代理到后端）

---

### RP-3: Footer 订阅更新功能

**文件**: `web/src/components/layout/Footer.tsx`

**问题**: 邮箱订阅输入框无功能。

**修改**: 添加 form submit，调用后端 `/api/waitlist` 接口。

```tsx
const handleSubscribe = async (e: FormEvent) => {
  e.preventDefault();
  const email = subscribeEmailRef.current?.value;
  if (!email) return;
  try {
    const res = await fetch('/api/waitlist', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, source: 'footer-subscribe' }),
    });
    if (res.ok) {
      alert('订阅成功！');
      subscribeEmailRef.current.value = '';
    }
  } catch (err) {
    alert('订阅失败，请稍后再试');
  }
};
```

**已有后端**: `POST /api/waitlist` 已实现，无需新建 API。

---

### RP-4: `/request` 页面创建

**问题**: `/request/` 路由不存在，点击"提交任务"返回 404。

**方案**: 创建 `web/src/app/request/page.tsx`，作为任务提交入口。

**设计**:
- 标题：提交任务
- 表单字段：任务标题（必填）、任务描述（必填，Textarea）、来源 URL（可选）、联系邮箱（必填）
- 提交到 `POST /api/tasks`（需要已登录）或 `POST /api/applications`（公开，无需登录）
- **决策点**：任务提交是否需要用户登录？
  - 方案 A（推荐）：无需登录，提交到 `/api/applications`，管理员在后台处理
  - 方案 B：需要登录，提交到 `/api/tasks`，有用户 session

**决定采用方案 A**：简化流程，用户无需注册直接提交，后台管理员处理。

```tsx
// web/src/app/request/page.tsx
export default function RequestPage() {
  return (
    <div className="container mx-auto py-16 px-4">
      <h1 className="text-3xl font-bold mb-8">提交任务</h1>
      <TaskRequestForm />
    </div>
  );
}
```

---

### RP-5: Footer 社交图标链接

**文件**: `web/src/components/layout/Footer.tsx`

**问题**: 社交图标 GH/X/DC/◎ 显示但无 href。

**修改**:
```tsx
<a href="https://github.com/147qaz258-ead/cozmio" target="_blank" rel="noopener" className="text-warm-600 hover:text-warm-900">GH</a>
<a href="https://x.com/wjnhng419090" target="_blank" rel="noopener" className="text-warm-600 hover:text-warm-900">X</a>
<a href="https://discord.gg/cozmio" target="_blank" rel="noopener" className="text-warm-600 hover:text-warm-900">DC</a>
```

---

### RP-6: Cloudflare Pages RSC 问题（环境问题，可选）

**问题**: Next.js `output: "export"` 在 Cloudflare Pages 上 RSC prefetch 404。

**方案选择**:
- **方案 A（推荐）**: 保持现状，不修。功能完全正常，只是预取失败，不影响用户体验。控制台错误不影响实际运行。
- **方案 B**: 迁移到 Cloudflare Pages with Next.js 运行时（需要 Cloudflare Pages 项目配置改用 Next.js runtime而非静态导出）。

**建议**: 采用方案 A，不修这个问题。降低复杂度。

---

### RP-7: `/use` 页面中文化

**文件**: `web/src/app/use/page.tsx` 和 `web/src/components/use/EmailLoginForm.tsx`

**问题**: 页面显示英文 "Welcome to Cozmio" / "Enter your email"，与整体中文风格不一致。

**修改**: 将所有英文文本替换为中文。

```tsx
// EmailLoginForm.tsx 修改
// 修改前
<h1 className="text-2xl font-bold text-warm-900 mb-2">Welcome to Cozmio</h1>
<p className="text-warm-600 mb-6">Enter your email to access your workspace</p>

// 修改后
<h1 className="text-2xl font-bold text-warm-900 mb-2">欢迎使用 Cozmio</h1>
<p className="text-warm-600 mb-6">输入邮箱以访问您的工作空间</p>
```

---

### RP-8: 下载链接 HTTPS 修复

**问题**: `http://47.76.116.209/api/downloads/latest` 在 HTTPS 页面被 block。

**方案**: 通过反向代理（RP-2）解决。前端请求 `/api/downloads/latest`，nginx 转发到后端。后端 R2 signed URL 生成保持不变。

**注意**: 需要后端在 `/api/downloads/latest` 支持 `platform` 参数查 Cloudflare R2。

---

## 三、任务依赖关系

```
RP-2 (nginx 反向代理) ← 解所有 API 问题
  ├── RP-3 (订阅功能) ← 依赖 RP-2
  ├── RP-4 (Request 页面) ← 依赖 RP-2
  ├── RP-7 (/use 中文化) ← 独立
  ├── RP-8 (下载链接) ← 依赖 RP-2
  │
RP-1 (Footer 链接) ← 独立
RP-5 (社交图标) ← 独立
RP-6 (RSC 问题) ← 可选，不修
```

---

## 四、验证清单

修复完成后，必须在部署环境验证以下内容：

```
[ ] Footer 产品列：点击"智能体" → /agents/ 页面
[ ] Footer 资源列：点击"案例" → /cases/ 页面
[ ] Footer 公司列：点击"关于我们" → /about/ 页面
[ ] 订阅更新：输入邮箱点击提交 → 显示成功提示（或 API 报错）
[ ] /request 页面：访问 `/request/` 显示任务提交表单
[ ] 提交任务：填写表单点击提交 → 后端收到数据（/api/applications）
[ ] /use 页面：输入邮箱 → 发送验证码 API 调用成功（nginx 代理）
[ ] 下载链接：点击"下载 Desktop App" → 触发 R2 signed URL 下载
[ ] 社交图标：点击 GH → GitHub 页面新标签页打开
[ ] 控制台：访问落地页各页面，无 RSC 404 阻断错误（RP-6 不修情况下）
```

---

## 五、实现优先级

1. **RP-1** — Footer 链接（5 分钟，阻断所有 footer 导航）
2. **RP-2** — nginx 反向代理配置（30 分钟，解决全部 API 问题）
3. **RP-3** — 订阅功能（15 分钟）
4. **RP-4** — /request 页面（30 分钟，核心功能）
5. **RP-5** — 社交图标（10 分钟）
6. **RP-7** — /use 中文化（10 分钟）
7. **RP-8** — 下载链接（依赖 RP-2）