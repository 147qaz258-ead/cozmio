# Cozmio V1 全栈核心设计

**版本：** 1.0
**日期：** 2026-05-06
**状态：** 设计完成，待实施

---

## 1. 整体架构

```
┌─────────────────────────────────────────────────────────────┐
│  Cloudflare Pages (Next.js 16)                              │
│  静态导出 → cozmio.net / www.cozmio.net                     │
│  前端调用 cozmio-api.example.com/api/*                      │
└──────────────────────┬──────────────────────────────────────┘
                       │ HTTPS
                       ▼
┌─────────────────────────────────────────────────────────────┐
│  cozmio-api (Node.js + Fastify)                             │
│  Docker 部署，端口 3000                                     │
│  - Session-based auth (email magic link)                   │
│  - REST API (统一响应格式)                                  │
│  - Resend (Transactional email)                            │
│  - R2/S3 (Signed URL for downloads)                        │
└──────────────────────┬──────────────────────────────────────┘
                       │
          ┌────────────┴────────────┐
          ▼                         ▼
┌─────────────────────┐    ┌─────────────────────┐
│  PostgreSQL 16       │    │  Cloudflare R2      │
│  cozmio_db           │    │  downloads bucket    │
│  Drizzle ORM         │    │  versioned files     │
└─────────────────────┘    └─────────────────────┘
```

---

## 2. 访问控制矩阵

| 功能 | 权限要求 |
|------|---------|
| `/use` 首页 | public（无需 session） |
| `/use/tasks` | web_access（email session） |
| `/use/tasks/[id]` | web_access（email session） |
| POST /api/tasks | web_access |
| GET /api/downloads/latest | public |
| GET /api/downloads/latest?platform=windows&access=beta | beta_access |
| 下载 Desktop Node 安装包 | desktop_access |
| 下载硬件节点软件 | hardware_access |
| `/admin/*` | admin role（独立登录） |

**Access 标志位（users 表）：**
- `web_access`: 始终 true（公开页面）
- `beta_access`: 默认 false，需 admin 审批后开启
- `desktop_access`: 默认 false，需 admin 邀请或 invite_code 激活
- `hardware_access`: 默认 false，需 admin 邀请或 invite_code 激活

---

## 3. 数据库 Schema（PostgreSQL + Drizzle）

### 3.1 users

```typescript
users = {
  id: uuid PK DEFAULT gen_random_uuid()
  email: text UNIQUE NOT NULL
  name: text
  role: text DEFAULT 'user'  // 'user' | 'admin'
  web_access: boolean DEFAULT true        // 始终 true
  beta_access: boolean DEFAULT false      // beta 版本下载权限
  desktop_access: boolean DEFAULT false    // Desktop Node 下载权限
  hardware_access: boolean DEFAULT false  // 硬件节点权限
  invite_code: text                        // 用户专属邀请码（可分享给其他人）
  created_at: timestamp DEFAULT now()
  updated_at: timestamp DEFAULT now()
}
```

### 3.2 sessions

```typescript
user_sessions = {
  id: uuid PK DEFAULT gen_random_uuid()
  user_id: uuid FK → users.id
  email: text NOT NULL
  token: text UNIQUE NOT NULL  // 6位数字验证码 or magic link token
  token_type: text NOT NULL    // 'otp' | 'magic_link'
  expires_at: timestamp NOT NULL
  created_at: timestamp DEFAULT now()
}
```

### 3.3 applications（内测申请）

```typescript
applications = {
  id: uuid PK
  name: text NOT NULL
  email: text NOT NULL
  company: text
  role: text
  use_case: text
  source: text
  status: text DEFAULT 'new'  // 'new' | 'reviewed' | 'invited' | 'rejected'
  admin_note: text
  created_at: timestamp DEFAULT now()
  updated_at: timestamp DEFAULT now()
}
```

### 3.4 waitlist

```typescript
waitlist = {
  id: uuid PK
  email: text UNIQUE NOT NULL
  name: text
  created_at: timestamp DEFAULT now()
}
```

### 3.5 tasks

```typescript
tasks = {
  id: uuid PK
  user_id: uuid FK → users.id  // 可为空（匿名提交）
  email: text NOT NULL
  title: text NOT NULL
  prompt: text NOT NULL
  source_url: text
  source_type: text  // 'manual' | 'copy' | 'import'
  status: text DEFAULT 'submitted'
  // submitted → queued → processing → needs_review → done
  //                                            ↘ failed
  //                                          ↘ cancelled
  result_summary: text
  result_payload: jsonb
  error_message: text
  internal_note: text           // admin 内部备注
  share_token: text UNIQUE      // 只读分享 token
  created_at: timestamp DEFAULT now()
  updated_at: timestamp DEFAULT now()
}
```

### 3.6 task_events

```typescript
task_events = {
  id: uuid PK
  task_id: uuid FK → tasks.id
  event_type: text NOT NULL
  // 'created' | 'status_change' | 'note_added' | 'email_sent' | 'result_added'
  message: text
  metadata: jsonb
  created_at: timestamp DEFAULT now()
}
```

### 3.7 download_versions

```typescript
download_versions = {
  id: uuid PK
  version: text NOT NULL
  platform: text NOT NULL  // 'windows' | 'macos' | 'linux'
  file_key: text NOT NULL  // R2/S3 object key
  changelog: text
  access_level: text DEFAULT 'public'  // 'public' | 'beta' | 'desktop' | 'hardware'
  is_latest: boolean DEFAULT false
  is_active: boolean DEFAULT true
  created_at: timestamp DEFAULT now()
  updated_at: timestamp DEFAULT now()
}
```

### 3.8 admin_sessions

```typescript
admin_sessions = {
  id: uuid PK DEFAULT gen_random_uuid()
  email: text NOT NULL
  token: text UNIQUE NOT NULL  // session token
  expires_at: timestamp NOT NULL
  created_at: timestamp DEFAULT now()
}
```

---

## 4. API 设计

### 4.1 统一响应格式

```typescript
// 成功
{ "ok": true, "data": { ... } }

// 失败
{ "ok": false, "error": { "code": "ERROR_CODE", "message": "Human readable" } }

// 分页
{ "ok": true, "data": [...], "pagination": { "page": 1, "pageSize": 20, "total": 100 } }
```

**错误码：**
| code | HTTP Status | 说明 |
|------|-------------|------|
| `VALIDATION_ERROR` | 400 | 参数校验失败 |
| `NOT_FOUND` | 404 | 资源不存在 |
| `UNAUTHORIZED` | 401 | 未认证（session 无效/过期）|
| `FORBIDDEN` | 403 | 权限不足（access_level 不够）|
| `RATE_LIMITED` | 429 | 请求过于频繁 |
| `INTERNAL_ERROR` | 500 | 服务器内部错误 |

---

### 4.2 Public API

#### POST /api/auth/send-code
发送邮箱验证码或 magic link

**Request:**
```json
{ "email": "user@example.com", "type": "otp" | "magic_link" }
```

**Response:**
```json
{ "ok": true, "data": { "message": "Verification code sent" } }
```

**行为：**
- email 存在 → 发送 OTP 到该邮箱
- email 不存在 → 自动创建 user 记录，再发送 OTP
- OTP 有效期 10 分钟
- 同一邮箱 60 秒内不能重复发送

#### POST /api/auth/verify
验证 OTP 或 magic link

**Request:**
```json
{ "email": "user@example.com", "token": "123456" }
```
或
```json
{ "magic_token": "xxx" }
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "user": { "id": "...", "email": "...", "role": "user" },
    "session_token": "uuid-session-token",
    "expires_at": "2026-05-06T12:00:00Z"
  }
}
```

**行为：**
- 设置 httpOnly cookie `session_token`，有效期 7 天
- OTP 验证后立即销毁（一次性使用）
- 返回用户信息 + session token

#### POST /api/auth/logout
登出

**Request:**（Cookie 自动携带）

**Response:**
```json
{ "ok": true, "data": { "message": "Logged out" } }
```

#### GET /api/auth/me
获取当前用户信息

**Request:**（Cookie 自动携带）

**Response:**
```json
{ "ok": true, "data": { "id": "...", "email": "...", "role": "user" } }
```
无 session 时返回 401。

---

#### POST /api/applications
提交内测申请

**Request:**
```json
{
  "name": "张三",
  "email": "zhang@example.com",
  "company": "某公司",
  "role": "CTO",
  "use_case": "想用来自动化代码审查",
  "source": "朋友推荐"
}
```

**Response:**
```json
{ "ok": true, "data": { "id": "...", "status": "new" } }
```

**邮件：** 触发 `application-received` 邮件模板

---

#### POST /api/waitlist
加入 waitlist

**Request:**
```json
{ "email": "...", "name": "..." }
```

---

#### GET /api/downloads/latest
获取最新稳定版下载信息

**Request Query:**
```
?platform=windows
```

**Response:**
```json
{
  "ok": true,
  "data": {
    "version": "1.0.0",
    "platform": "windows",
    "download_url": "https://cozmio-api.example.com/api/downloads/file?key=...",
    "changelog": "...",
    "file_size": "...",
    "access_level": "public"
  }
}
```

**行为：**
- 查找 `is_latest=true AND is_active=true AND platform=?` 的记录
- 生成 R2 签名 URL（有效期 15 分钟）
- 无需登录（public access）

---

### 4.3 User API（需要 session）

所有 User API 需要请求头携带 `Authorization: Bearer <session_token>` 或 Cookie。

#### GET /api/tasks
获取当前用户的任务列表

**Request Query:**
```
?page=1&pageSize=20&status=processing
```

**Response:**
```json
{
  "ok": true,
  "data": [
    {
      "id": "...",
      "title": "...",
      "status": "processing",
      "created_at": "...",
      "updated_at": "..."
    }
  ],
  "pagination": { "page": 1, "pageSize": 20, "total": 45 }
}
```

#### POST /api/tasks
创建新任务

**Request:**
```json
{
  "title": "帮我分析销售数据",
  "prompt": "我有一个 CSV 文件，包含...，请帮我...",
  "source_url": "",
  "source_type": "manual"
}
```

**Response:**
```json
{ "ok": true, "data": { "id": "...", "status": "submitted" } }
```

**邮件：** 触发 `task-received` 确认邮件（可选，V1 先不做）

#### GET /api/tasks/:id
获取任务详情

**Response:**
```json
{
  "ok": true,
  "data": {
    "id": "...",
    "title": "...",
    "prompt": "...",
    "status": "done",
    "result_summary": "分析完成，共发现 3 个关键趋势...",
    "result_payload": { "...Raw result data..." },
    "error_message": null,
    "created_at": "...",
    "updated_at": "...",
    "share_token": "readonly-access-token"
  }
}
```

**行为：**
- `user_id` 匹配当前 session user 或拥有 `share_token` 可访问

---

### 4.4 Admin API（需要 admin session）

#### POST /api/admin/login
Admin 登录

**Request:**
```json
{ "email": "admin@cozmio.net", "password": "..." }
```

**Response:**
```json
{ "ok": true, "data": { "session_token": "...", "expires_at": "..." } }
```

**行为：**
- 验证 `ADMIN_EMAIL` + `ADMIN_PASSWORD_HASH`
- 设置 httpOnly cookie `admin_session`
- Session 有效期 24 小时

#### POST /api/admin/logout
Admin 登出

#### GET /api/admin/me
获取当前 admin 信息

---

#### GET /api/admin/applications
申请列表

**Request Query:**
```
?page=1&pageSize=20&status=new&search=关键词
```

**Response:**
```json
{
  "ok": true,
  "data": [...],
  "pagination": { "page": 1, "pageSize": 20, "total": 100 }
}
```

#### GET /api/admin/applications/:id
申请详情

#### PATCH /api/admin/applications/:id
更新申请状态

**Request:**
```json
{ "status": "invited", "admin_note": "已发送邀请邮件" }
```

**邮件：** status=invited 时触发 `application-invited` 邮件

---

#### GET /api/admin/tasks
任务列表

**Request Query:**
```
?page=1&pageSize=20&status=processing&user_id=xxx&search=关键词
```

#### GET /api/admin/tasks/:id
任务详情（含 task_events）

#### PATCH /api/admin/tasks/:id
更新任务

**Request:**
```json
{
  "status": "done",
  "result_summary": "已完成分析",
  "result_payload": { "summary": "..." },
  "internal_note": "内部备注"
}
```

**邮件：** status 变更时触发 `task-status-update` 邮件

---

#### GET /api/admin/users
用户列表

**Request Query:**
```
?page=1&pageSize=20&search=email
```

#### GET /api/admin/users/:id
用户详情（含该用户所有 tasks）

#### PATCH /api/admin/users/:id
更新用户 access 权限

**Request:**
```json
{
  "beta_access": true,
  "desktop_access": true,
  "invite_code": "CUSTOMCODE"
}
```

**邮件：** access 变更时触发对应邀请邮件

---

#### GET /api/admin/downloads
下载版本列表

#### POST /api/admin/downloads
创建新版本

**Request:**
```json
{
  "version": "1.1.0",
  "platform": "windows",
  "file": "<binary>",
  "changelog": "...",
  "access_level": "beta",
  "is_latest": true
}
```

**行为：**
- 上传文件到 R2
- 保存 file_key 到数据库
- 如果 `is_latest=true`，则将该 platform 的其他版本 `is_latest=false`

#### PATCH /api/admin/downloads/:id
更新版本信息

---

## 5. 邮件系统（Resend）

### 5.1 邮件模板

| Template ID | 触发场景 | 收件人 |
|-------------|---------|--------|
| `application-received` | 申请提交成功 | 申请人 |
| `application-invited` | 申请被邀请 | 申请人 |
| `auth-otp` | 发送登录验证码 | 用户 |
| `auth-magic-link` | 发送魔法链接 | 用户 |
| `task-status-update` | 任务状态变更 | 任务提交者 |
| `invite-beta-access` | 获得 beta 权限 | 用户 |
| `invite-desktop-access` | 获得 desktop 权限 | 用户 |
| `invite-hardware-access` | 获得 hardware 权限 | 用户 |

### 5.2 邮件内容要求

- 发件人：`Cozmio <noreply@cozmio.net>`
- 支持中文
- 包含 Cozmio branding
- 邮件内链接指向 cozmio.net 对应页面

---

## 6. 页面设计

### 6.1 /use（使用页面）

**URL：** `https://cozmio.net/use`

**功能模块：**

#### 6.1.1 Email 登录区（未登录时显示）
- Email 输入框
- "发送验证码" 按钮
- 倒计时（60 秒后可重发）
- 验证码输入（6 位数字）
- "验证并进入" 按钮
- Loading / Error 状态

**用户体验流程：**
1. 用户输入 email → 点击"发送验证码"
2. 收到 6 位数字验证码邮件
3. 输入验证码 → 点击"验证并进入"
4. 验证成功 → 创建 session → 进入 workspace

#### 6.1.2 Workspace（登录后显示）

**顶部导航：**
- Logo + "Cozmio Use"
- 用户 email + 下拉菜单（退出登录）
- 移动端：折叠菜单

**主内容区：**

**Tab 1: 提交任务（默认）**
- 标题输入（必填，max 200 chars）
- 任务描述 textarea（必填，max 5000 chars）
- 来源 URL（可选）
- 提交按钮
- Success: 显示"任务已提交，ID: xxx"，清空表单
- Error: 显示错误信息

**Tab 2: 任务历史**
- 列表形式展示用户的 tasks
- 每行：标题 + 状态标签 + 创建时间 + 更新时间
- 状态标签颜色：
  - submitted: 灰
  - queued: 蓝
  - processing: 橙
  - needs_review: 紫
  - done: 绿
  - failed: 红
  - cancelled: 灰
- 点击一行 → 跳转 task detail
- Empty 状态：显示引导用户提交第一个任务

**Tab 3: 任务详情（点击任务后）**
- 返回按钮
- 任务标题（大字）
- 状态标签 + 时间
- prompt 内容（可复制）
- result_summary（如果有）
- result_payload（JSON 展开，如果有）
- error_message（如果有，红色背景）
- 内部笔记（admin 添加的，仅 admin 可见，V1 隐藏）

**通用状态：**
- Loading: 骨架屏或 spinner
- Error: 重试按钮 + 错误信息
- Empty: 引导图 + 提示文字

---

### 6.2 /use/tasks/[id]（任务详情页）

**URL：** `https://cozmio.net/use/tasks/[id]`

**需要 session 或 share_token 验证**

**内容：** 同 6.1.2 的任务详情

---

### 6.3 /admin（管理员后台）

**URL：** `https://cozmio.net/admin`

**需要 admin session**

#### 6.3.1 /admin/login（登录页）
- Email + 密码表单
- 登录按钮
- 记住我（30 天有效期）
- Error 状态

#### 6.3.2 /admin/dashboard（主面板）
- 统计卡片：用户数、申请数、任务数、下载版本数
- 最近新申请（5 条）
- 最近新任务（5 条）
- 系统状态

#### 6.3.3 /admin/applications（申请管理）
- 搜索框（按 name/email/company 搜索）
- 状态筛选 tabs（全部 / new / reviewed / invited / rejected）
- 表格列表：
  - ID / Name / Email / Company / Status / Date / Actions
- 点击行 → 展开详情
- Actions：查看详情、修改状态、添加备注
- 详情弹窗/页面：
  - 完整信息展示
  - Status 修改下拉
  - Admin note 文本框
  - 保存按钮

#### 6.3.4 /admin/tasks（任务管理）
- 搜索框（按 title/email 搜索）
- 状态筛选 tabs
- 表格列表：
  - ID / Title / Email / Status / Created / Updated / Actions
- 点击 → 展开详情
- 详情：
  - 完整 task 信息
  - result_summary 编辑框
  - result_payload JSON 编辑器
  - internal_note 文本框（admin 内部使用）
  - status 修改下拉
  - task_events 时间线
  - 保存

#### 6.3.5 /admin/users（用户管理）
- 搜索框（按 email/name）
- 表格：
  - ID / Email / Name / Role / Access Flags / Created / Actions
- 点击 → 详情
- 编辑：
  - beta_access toggle
  - desktop_access toggle
  - hardware_access toggle
  - invite_code 文本框
  - 保存

#### 6.3.6 /admin/downloads（下载版本管理）
- 版本列表（表格）
  - Version / Platform / Access Level / Latest / Active / Created / Actions
- 添加新版本按钮
- 添加/编辑弹窗：
  - Version 文本框
  - Platform 下拉
  - Access Level 下拉
  - Changelog textarea
  - 文件上传
  - Is Latest checkbox
  - Is Active checkbox

---

## 7. 技术选型

| 层级 | 技术 | 版本 |
|------|------|------|
| 前端框架 | Next.js | 16.2.x |
| 后端框架 | Fastify | 5.x |
| 数据库 | PostgreSQL | 16 |
| ORM | Drizzle ORM | latest |
| 邮件 | Resend | latest |
| 文件存储 | Cloudflare R2 | — |
| 容器化 | Docker + Docker Compose | latest |
| 部署目标 | VPS + Docker Compose | — |

---

## 8. 目录结构

```
cozmio/
├── web/                          # Next.js 前端（现有）
│   ├── src/
│   │   ├── app/
│   │   │   ├── use/
│   │   │   │   ├── page.tsx         # /use 主页
│   │   │   │   └── tasks/
│   │   │   │       ├── page.tsx     # /use/tasks 列表
│   │   │   │       └── [id]/
│   │   │   │           └── page.tsx # /use/tasks/:id 详情
│   │   │   └── admin/
│   │   │       ├── page.tsx          # /admin 登录
│   │   │       ├── dashboard/
│   │   │       ├── applications/
│   │   │       ├── tasks/
│   │   │       ├── users/
│   │   │       └── downloads/
│   │   ├── lib/
│   │   │   ├── api.ts                # API client（调用后端）
│   │   │   └── site-config.ts        # 站点配置
│   │   └── components/
│   │       └── admin/                # Admin 专用组件
│   ├── package.json
│   └── ...
│
├── cozmio-api/                   # 新建：后端 API 服务
│   ├── src/
│   │   ├── index.ts                # 入口
│   │   ├── app.ts                  # Fastify 实例
│   │   ├── db/
│   │   │   ├── index.ts             # Drizzle client
│   │   │   ├── schema.ts            # Schema 定义
│   │   │   └── migrations/          # SQL migrations
│   │   ├── routes/
│   │   │   ├── auth.ts              # /api/auth/*
│   │   │   ├── applications.ts       # /api/applications
│   │   │   ├── waitlist.ts          # /api/waitlist
│   │   │   ├── downloads.ts          # /api/downloads
│   │   │   ├── tasks.ts             # /api/tasks
│   │   │   └── admin/
│   │   │       ├── index.ts         # admin 路由注册
│   │   │       ├── auth.ts          # /api/admin/login, logout, me
│   │   │       ├── applications.ts  # /api/admin/applications
│   │   │       ├── tasks.ts         # /api/admin/tasks
│   │   │       ├── users.ts         # /api/admin/users
│   │   │       └── downloads.ts     # /api/admin/downloads
│   │   ├── services/
│   │   │   ├── email.ts             # Resend 服务
│   │   │   ├── storage.ts           # R2 服务
│   │   │   └── session.ts           # Session 管理
│   │   ├── middleware/
│   │   │   ├── require-auth.ts      # 用户认证中间件
│   │   │   └── require-admin.ts    # Admin 认证中间件
│   │   └── lib/
│   │       ├── response.ts          # 统一响应格式
│   │       └── errors.ts            # 错误类型定义
│   ├── drizzle.config.ts
│   ├── package.json
│   ├── tsconfig.json
│   ├── Dockerfile
│   ├── docker-compose.yml
│   ├── .env.example
│   └── seeds/
│       └── seed.ts                   # 测试数据种子
│
└── docs/superpowers/specs/
    └── 2026-05-06-cozmio-v1-backend-design.md  # 本文档
```

---

## 9. 环境变量

### cozmio-api .env.example

```bash
# Database
DATABASE_URL=postgresql://cozmio:password@localhost:5432/cozmio_db

# Auth
ADMIN_EMAIL=admin@cozmio.net
ADMIN_PASSWORD_HASH=bcrypt_hash_of_password
SESSION_SECRET=random_32_char_string

# Email (Resend)
RESEND_API_KEY=re_xxxxx

# R2 / S3
R2_ACCOUNT_ID=your_account_id
R2_ACCESS_KEY_ID=your_access_key
R2_SECRET_ACCESS_KEY=your_secret_key
R2_BUCKET=cozmio-downloads
R2_PUBLIC_URL=https://downloads.cozmio.net

# 可选：如果用 AWS S3
# S3_ENDPOINT=https://s3.amazonaws.com
# S3_REGION=us-east-1

# CORS（前端域名）
CORS_ORIGIN=https://cozmio.net

# API Base URL（用于生成 magic link）
API_BASE_URL=https://api.cozmio.net
```

### web .env.local（新增）

```bash
NEXT_PUBLIC_API_BASE_URL=https://api.cozmio.net
```

---

## 10. 部署说明

### 10.1 本地开发

```bash
# 1. 启动数据库和 API
cd cozmio-api
cp .env.example .env
# 编辑 .env 填入真实值
docker-compose up -d postgres
npx drizzle-kit migrate
npx ts-node seeds/seed.ts
npm run dev

# 2. 前端开发
cd web
cp .env.example .env.local
npm run dev
```

### 10.2 生产部署（VPS + Docker Compose）

```bash
# 1. 上传代码到 VPS
git clone https://github.com/147qaz258-ead/cozmio.git
cd cozmio/cozmio-api

# 2. 配置环境变量
cp .env.example .env
vim .env  # 填入真实值

# 3. 启动
docker-compose -f docker-compose.yml up -d

# 4. 运行 migration
docker exec cozmio-api-1 npx drizzle-kit migrate --env production

# 5. 初始化测试数据
docker exec cozmio-api-1 npx ts-node seeds/seed.ts

# 查看日志
docker logs -f cozmio-api-1

# 备份数据库
docker exec cozmio-postgres-1 pg_dump -U cozmio cozmio_db > backup_$(date +%Y%m%d).sql
```

### 10.3 Coolify 兼容

Docker Compose 格式兼容 Coolify 平台导入，无需修改即可部署。

---

## 11. 测试计划

### 11.1 API 测试

- [ ] POST /api/auth/send-code → 发送验证码
- [ ] POST /api/auth/verify → 验证并登录
- [ ] GET /api/auth/me → 获取当前用户
- [ ] POST /api/tasks → 创建任务
- [ ] GET /api/tasks → 获取任务列表（分页）
- [ ] GET /api/tasks/:id → 获取任务详情
- [ ] Admin login → 登录
- [ ] Admin CRUD → 完整 admin 操作

### 11.2 前端测试

- [ ] /use 未登录 → 显示 email 登录表单
- [ ] /use 登录成功 → 进入 workspace
- [ ] /use 提交任务 → 成功反馈
- [ ] /use 任务历史 → 列表展示
- [ ] /use 任务详情 → 完整信息展示
- [ ] /admin 登录 → 成功
- [ ] /admin dashboard → 统计数据
- [ ] /admin applications → CRUD
- [ ] /admin tasks → CRUD + 状态修改
- [ ] /admin users → access 修改
- [ ] /admin downloads → 版本管理

---

## 12. 已知限制与未来扩展

### V1 不做
- 多语言（i18n）
- 深色模式
- 完整移动端原生体验
- 团队/workspace 概念
- 计费系统
- WebSocket 实时状态推送
- OAuth 第三方登录
- API Key 体系

### 预留扩展点
- `task_events` 表可扩展为完整审计日志
- `users.invite_code` 可实现邀请奖励机制
- `download_versions.access_level` 可扩展更多权限等级
- `sessions.token_type` 可扩展 remember_me（长期 token）
- `tasks.result_payload` JSONB 可存储任意结构化结果

---

## 13. 设计决策记录

| 决策 | 理由 |
|------|------|
| Email OTP 而非密码登录 | V1 最简体验，用户无需记忆密码 |
| Fastify 而非 Express | 更好的 TypeScript 支持，更快的性能 |
| Drizzle 而非 Prisma | 轻量、SQL-like query、migration 简单 |
| R2 而非 S3 | Cloudflare 生态一致，免费额度充足 |
| Session Cookie 而非 JWT | 简单、可撤销、适合本场景 |
| Magic link 作为补充 | 适合移动端，减少输入 |