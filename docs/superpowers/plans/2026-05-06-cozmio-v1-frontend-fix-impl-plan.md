# Cozmio V1 前端修复实施计划

**版本：** 1.0
**日期：** 2026-05-06
**基于：** `docs/superpowers/specs/2026-05-06-cozmio-v1-frontend-fix-design.md`
**状态：** 已锁定 ✓

---

## 当前真相 (Current Truth)

### 已验证文件

- `web/src/components/layout/Footer.tsx:1-67` — Footer 链接已使用 `<Link>` 和 `<a>` 标签，可点击。**A4 设计文档描述失实**。但缺少：订阅表单 handler、社交图标（X/Discord）。
- `web/src/app/request/page.tsx:1-5` — `/request` 页面已存在，引用 `RequestPage` 组件。**A1 设计文档描述失实**。
- `web/src/components/use/EmailLoginForm.tsx` — 需检查是否中文化。
- `web/src/lib/site-config.ts:1-66` — 下载链接使用 `${API_BASE}/api/downloads/latest?platform=windows`，API_BASE 来自 `process.env.NEXT_PUBLIC_API_BASE_URL`。

---

## Implementation Shape

### RP-1: Footer 订阅 + 社交图标

**文件**: `web/src/components/layout/Footer.tsx`

**当前真实现状** (实测 at line 53-62):
```tsx
// 底部只有 email + GitHub 两个链接
<div className="flex flex-wrap gap-4">
  <a href={siteConfig.links.email}>{siteConfig.email}</a>
  <a href={siteConfig.links.github} target="_blank" rel="noreferrer">{t.nav.github}</a>
</div>
```

**修改为**:
```tsx
// 1. 导入 useRef, FormEvent
import { useRef, FormEvent } from "react";

// 2. 添加订阅 handler
const subscribeEmailRef = useRef<HTMLInputElement>(null);

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
    } else {
      alert('订阅失败，请稍后再试');
    }
  } catch {
    alert('订阅失败，请稍后再试');
  }
};

// 3. 在 footer 左侧 logo 下方添加订阅表单（description 之后）
<form onSubmit={handleSubscribe} className="mt-4 flex gap-2">
  <input
    ref={subscribeEmailRef}
    type="email"
    placeholder="输入邮箱订阅更新"
    className="flex-1 rounded border border-black/10 px-3 py-1.5 text-sm"
    required
  />
  <button type="submit" className="rounded bg-primary px-3 py-1.5 text-sm text-white">
    订阅
  </button>
</form>

// 4. 社交图标区域添加 X 和 Discord
<div className="flex flex-wrap gap-4">
  <a href={siteConfig.links.email} className="transition-colors hover:text-primary-text">
    {siteConfig.email}
  </a>
  <a href={siteConfig.links.github} target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
    {t.nav.github}
  </a>
  <a href="https://x.com/wjnhng419090" target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
    X
  </a>
  <a href="https://discord.gg/cozmio" target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
    DC
  </a>
</div>
```

**验证**: 构建后检查 Footer 无 TypeScript 错误

---

### RP-2: `/use` 页面中文化

**文件**: `web/src/app/use/page.tsx:22-23` + `web/src/components/use/EmailLoginForm.tsx`

**当前真实现状** (use/page.tsx):
```tsx
<h1 className="text-2xl font-bold text-warm-900 mb-2">Welcome to Cozmio</h1>   // line 22
<p className="text-warm-600 mb-6">Enter your email to access your workspace</p>  // line 23
```

**修改为** (use/page.tsx):
```tsx
<h1 className="text-2xl font-bold text-warm-900 mb-2">欢迎使用 Cozmio</h1>
<p className="text-warm-600 mb-6">输入邮箱以访问您的工作空间</p>
```

**当前真实现状** (EmailLoginForm.tsx line 45-94):
```tsx
// step === "email" 时：
placeholder="your@email.com"          // line 53
"Sending..." / "Continue"             // line 62

// step === "code" 时：
"Check your email..."                 // line 67
placeholder="123456"                  // line 72
"Verifying..." / "Verify"             // line 82
"Use a different email"              // line 89
```

**修改为** (EmailLoginForm.tsx):
```tsx
// step === "email" 时：
placeholder="输入邮箱地址"
"发送中..." / "继续"

// step === "code" 时：
"请查收邮件中的验证码"
placeholder="000000"
"验证中..." / "确认"
"使用其他邮箱"
```

**验证**: 本地 `npm run dev` 访问 `/use` 确认全中文显示

---

### RP-3: 前端 API_BASE_URL 配置

**文件**: `web/.env.production`（新建）

**问题**: 当前 `site-config.ts` 使用 `process.env.NEXT_PUBLIC_API_BASE_URL || "https://api.cozmio.net"`，生产环境指向不存在的 `api.cozmio.net`。

**修改为**:
```
NEXT_PUBLIC_API_BASE_URL=/api
```

这样前端所有 API 请求变为同源 `/api/*`，通过 nginx 代理到后端。

---

### RP-4: nginx 反向代理配置（服务器端）

**目标服务器**: `root@47.76.116.209`

**操作**: 通过 SSH 在服务器上配置 nginx。

```bash
# 检查 nginx 是否安装
nginx -v

# 如果没有，安装
apt install nginx

# 创建站点配置
cat > /etc/nginx/sites-available/cozmio << 'EOF'
server {
    listen 80;
    server_name cozmio.net www.cozmio.net;

    location /api/ {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }

    location / {
        root /var/www/cozmio/out;
        try_files $uri $uri/ /index.html;
    }
}
EOF

# 启用站点
ln -sf /etc/nginx/sites-available/cozmio /etc/nginx/sites-enabled/

# 测试并重载
nginx -t && nginx -s reload
```

---

## 任务依赖关系

```
RP-3 (前端 .env.production) ← 独立
RP-4 (nginx 代理) ← 独立
RP-1 (Footer) ← 独立
RP-2 (/use 中文化) ← 独立
```

**RP-3 + RP-4 完成后**，以下功能自动生效：
- 订阅功能（RP-1 依赖 RP-3/RP-4）
- `/api/waitlist` 调用
- `/api/downloads/latest` 调用
- `/use` 页面 Email 登录 API

---

## 实施顺序

1. **RP-1**: 修改 Footer.tsx（订阅 + 社交图标）
2. **RP-2**: 修改 EmailLoginForm.tsx（中文化）
3. **RP-3**: 创建 `web/.env.production`
4. **RP-4**: 服务器配置 nginx 反向代理
5. **重新构建 + 重新部署前端** `npm run build`
6. **重启后端**（确保在 localhost:3000 运行）

---

## Risk → Verification Mapping

| Risk | 验证命令 | 预期结果 |
|------|---------|--------|
| 订阅 API 失败 | `curl -X POST http://47.76.116.209/api/waitlist -d '{"email":"test@test.com","source":"footer"}'` | `{"ok":true}` |
| nginx 代理不通 | `curl http://cozmio.net/api/health` | `{"ok":true}` |
| 下载链接被 block | 访问 https://cozmio.net/desktop/ 点击下载 | 触发下载（非 block） |
| /use 页面中文化 | 访问 https://cozmio.net/use/ | 全中文，无英文 "Welcome" |

---

## 计划保存

`docs/superpowers/plans/2026-05-06-cozmio-v1-frontend-fix-impl-plan.md`