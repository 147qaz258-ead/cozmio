# Cozmio V1 实施方案

**版本：** 1.0
**日期：** 2026-05-06
**状态：** 已锁定 ✓
**设计文档：** `docs/superpowers/specs/2026-05-06-cozmio-v1-backend-design.md`

---

## 实施顺序

```
Slice 1 (API 脚手架 + Schema)
    ↓
Slice 2 (User Email Auth)
    ↓
Slice 3 (User Tasks API)
    ↓
Slice 4 (/use 前端页面)
    ↓
Slice 5 (Admin Auth + Middleware)
    ↓
Slice 6 (Admin CRUD APIs)
    ↓
Slice 7 (/admin 前端页面)
    ↓
Slice 8 (Public APIs + 集成测试)
```

---

## Slice 1: cozmio-api 脚手架 + 数据库 Schema

**用户可见结果**：后端服务可启动，数据库表已创建，`/health` 返回 `{ "ok": true }`

### 新建文件

```
cozmio-api/
├── package.json
├── tsconfig.json
├── drizzle.config.ts
├── .env.example
├── Dockerfile
├── docker-compose.yml
└── src/
    ├── index.ts
    ├── app.ts
    ├── db/
    │   ├── index.ts
    │   ├── schema.ts
    │   └── migrations/.gitkeep
    └── lib/
        ├── response.ts
        └── errors.ts
```

### package.json 依赖

```json
{
  "name": "cozmio-api",
  "version": "1.0.0",
  "scripts": {
    "dev": "tsx watch src/index.ts",
    "build": "tsc",
    "start": "node dist/index.js",
    "db:generate": "drizzle-kit generate",
    "db:migrate": "drizzle-kit migrate",
    "db:push": "drizzle-kit push",
    "db:studio": "drizzle-kit studio"
  },
  "dependencies": {
    "fastify": "^5.2.0",
    "@fastify/cors": "^10.0.0",
    "@fastify/cookie": "^10.0.0",
    "drizzle-orm": "^0.38.0",
    "postgres": "^3.4.0",
    "resend": "^4.0.0",
    "@aws-sdk/client-s3": "^3.0.0",
    "@aws-sdk/s3-request-presigner": "^3.0.0",
    "bcrypt": "^5.1.0",
    "uuid": "^11.0.0"
  },
  "devDependencies": {
    "drizzle-kit": "^0.30.0",
    "tsx": "^4.0.0",
    "typescript": "^5.0.0",
    "@types/bcrypt": "^5.0.0",
    "@types/uuid": "^10.0.0"
  }
}
```

### drizzle.config.ts

```typescript
import { defineConfig } from "drizzle-kit";

export default defineConfig({
  schema: "./src/db/schema.ts",
  out: "./src/db/migrations",
  dialect: "postgresql",
  dbCredentials: {
    url: process.env.DATABASE_URL!,
  },
});
```

### docker-compose.yml

```yaml
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      DATABASE_URL: postgresql://cozmio:password@postgres:5432/cozmio_db
      RESEND_API_KEY: ${RESEND_API_KEY}
      R2_ACCOUNT_ID: ${R2_ACCOUNT_ID}
      R2_ACCESS_KEY_ID: ${R2_ACCESS_KEY_ID}
      R2_SECRET_ACCESS_KEY: ${R2_SECRET_ACCESS_KEY}
      R2_BUCKET: ${R2_BUCKET}
      R2_PUBLIC_URL: ${R2_PUBLIC_URL}
      ADMIN_EMAIL: ${ADMIN_EMAIL}
      ADMIN_PASSWORD_HASH: ${ADMIN_PASSWORD_HASH}
      SESSION_SECRET: ${SESSION_SECRET}
      CORS_ORIGIN: ${CORS_ORIGIN}
      API_BASE_URL: ${API_BASE_URL}
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: cozmio
      POSTGRES_PASSWORD: password
      POSTGRES_DB: cozmio_db
    volumes:
      - postgres_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U cozmio -d cozmio_db"]
      interval: 5s
      timeout: 5s
      retries: 5

volumes:
  postgres_data:
```

### src/lib/response.ts

```typescript
export function ok<T>(data: T) {
  return { ok: true, data };
}

export function error(code: string, message: string, status = 400) {
  return {
    status,
    body: { ok: false, error: { code, message } },
  };
}

export function paginated<T>(data: T[], page: number, pageSize: number, total: number) {
  return {
    ok: true,
    data,
    pagination: { page, pageSize, total },
  };
}
```

### src/lib/errors.ts

```typescript
export class AppError extends Error {
  constructor(
    public code: string,
    message: string,
    public status = 400
  ) {
    super(message);
  }
}

export const ERRORS = {
  VALIDATION_ERROR: (msg: string) => new AppError("VALIDATION_ERROR", msg, 400),
  NOT_FOUND: (msg: string) => new AppError("NOT_FOUND", msg, 404),
  UNAUTHORIZED: (msg = "Unauthorized") => new AppError("UNAUTHORIZED", msg, 401),
  FORBIDDEN: (msg = "Forbidden") => new AppError("FORBIDDEN", msg, 403),
  RATE_LIMITED: (msg = "Too many requests") => new AppError("RATE_LIMITED", msg, 429),
  INTERNAL_ERROR: (msg = "Internal server error") => new AppError("INTERNAL_ERROR", msg, 500),
} as const;
```

### src/db/schema.ts（完整 Drizzle Schema）

```typescript
import { pgTable, uuid, text, timestamp, boolean, jsonb, pgEnum } from "drizzle-orm/pg-core";
import { nanoid } from "nanoid";

// Enums
export const userRoleEnum = pgEnum("user_role", ["user", "admin"]);
export const tokenTypeEnum = pgEnum("token_type", ["otp", "magic_link"]);
export const applicationStatusEnum = pgEnum("application_status", ["new", "reviewed", "invited", "rejected"]);
export const taskStatusEnum = pgEnum("task_status", ["submitted", "queued", "processing", "needs_review", "done", "failed", "cancelled"]);
export const taskEventTypeEnum = pgEnum("task_event_type", ["created", "status_change", "note_added", "email_sent", "result_added"]);
export const platformEnum = pgEnum("platform", ["windows", "macos", "linux"]);
export const accessLevelEnum = pgEnum("access_level", ["public", "beta", "desktop", "hardware"]);

// Tables
export const users = pgTable("users", {
  id: uuid("id").defaultRandom().primaryKey(),
  email: text("email").unique().notNull(),
  name: text("name"),
  role: userRoleEnum("role").default("user").notNull(),
  webAccess: boolean("web_access").default(true).notNull(),
  betaAccess: boolean("beta_access").default(false).notNull(),
  desktopAccess: boolean("desktop_access").default(false).notNull(),
  hardwareAccess: boolean("hardware_access").default(false).notNull(),
  inviteCode: text("invite_code").unique(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const sessions = pgTable("sessions", {
  id: uuid("id").defaultRandom().primaryKey(),
  userId: uuid("user_id").references(() => users.id, { onDelete: "cascade" }),
  email: text("email").notNull(),
  token: text("token").unique().notNull(),
  tokenType: tokenTypeEnum("token_type").notNull(),
  expiresAt: timestamp("expires_at").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const applications = pgTable("applications", {
  id: uuid("id").defaultRandom().primaryKey(),
  name: text("name").notNull(),
  email: text("email").notNull(),
  company: text("company"),
  role: text("role"),
  useCase: text("use_case"),
  source: text("source"),
  status: applicationStatusEnum("status").default("new").notNull(),
  adminNote: text("admin_note"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const waitlist = pgTable("waitlist", {
  id: uuid("id").defaultRandom().primaryKey(),
  email: text("email").unique().notNull(),
  name: text("name"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const tasks = pgTable("tasks", {
  id: uuid("id").defaultRandom().primaryKey(),
  userId: uuid("user_id").references(() => users.id, { onDelete: "set null" }),
  email: text("email").notNull(),
  title: text("title").notNull(),
  prompt: text("prompt").notNull(),
  sourceUrl: text("source_url"),
  sourceType: text("source_type").default("manual"),
  status: taskStatusEnum("status").default("submitted").notNull(),
  resultSummary: text("result_summary"),
  resultPayload: jsonb("result_payload"),
  errorMessage: text("error_message"),
  internalNote: text("internal_note"),
  shareToken: text("share_token").unique().default(() => nanoid(16)),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const taskEvents = pgTable("task_events", {
  id: uuid("id").defaultRandom().primaryKey(),
  taskId: uuid("task_id").references(() => tasks.id, { onDelete: "cascade" }).notNull(),
  eventType: taskEventTypeEnum("event_type").notNull(),
  message: text("message"),
  metadata: jsonb("metadata"),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});

export const downloadVersions = pgTable("download_versions", {
  id: uuid("id").defaultRandom().primaryKey(),
  version: text("version").notNull(),
  platform: platformEnum("platform").notNull(),
  fileKey: text("file_key").notNull(),
  changelog: text("changelog"),
  accessLevel: accessLevelEnum("access_level").default("public").notNull(),
  isLatest: boolean("is_latest").default(false).notNull(),
  isActive: boolean("is_active").default(true).notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
  updatedAt: timestamp("updated_at").defaultNow().notNull(),
});

export const adminSessions = pgTable("admin_sessions", {
  id: uuid("id").defaultRandom().primaryKey(),
  email: text("email").notNull(),
  token: text("token").unique().notNull(),
  expiresAt: timestamp("expires_at").notNull(),
  createdAt: timestamp("created_at").defaultNow().notNull(),
});
```

### src/app.ts

```typescript
import Fastify from "fastify";
import cors from "@fastify/cors";
import cookie from "@fastify/cookie";

export async function buildApp() {
  const app = Fastify({ logger: true });

  await app.register(cors, {
    origin: process.env.CORS_ORIGIN?.split(",") || ["http://localhost:3000"],
    credentials: true,
  });

  await app.register(cookie, {
    parseOptions: {},
  });

  return app;
}
```

### src/index.ts

```typescript
import { buildApp } from "./app.js";
import { db } from "./db/index.js";

const app = await buildApp();

app.get("/health", async () => ({ ok: true }));

// 注册路由
app.register(import("./routes/health.js"), { prefix: "/api" });
app.register(import("./routes/auth.js"), { prefix: "/api/auth" });
app.register(import("./routes/applications.js"), { prefix: "/api/applications" });
app.register(import("./routes/waitlist.js"), { prefix: "/api/waitlist" });
app.register(import("./routes/downloads.js"), { prefix: "/api/downloads" });
app.register(import("./routes/tasks.js"), { prefix: "/api/tasks" });
app.register(import("./routes/admin/index.js"), { prefix: "/api/admin" });

// 全局错误处理
app.setErrorHandler((error, request, reply) => {
  if (error instanceof AppError) {
    return reply.status(error.status).send({ ok: false, error: { code: error.code, message: error.message } });
  }
  request.log.error(error);
  return reply.status(500).send({ ok: false, error: { code: "INTERNAL_ERROR", message: "Internal server error" } });
});

const port = Number(process.env.PORT) || 3000;
await app.listen({ port, host: "0.0.0.0" });
console.log(`Server listening on port ${port}`);
```

### 验证

```bash
cd cozmio-api
npm install
cp .env.example .env
# 填入 DATABASE_URL=postgresql://cozmio:password@localhost:5432/cozmio_db
docker-compose up -d postgres
npx drizzle-kit generate
npx drizzle-kit migrate
npm run dev &
sleep 3
curl http://localhost:3000/health
# → { "ok": true }
```

**状态**：已锁定 ✓

---

## Slice 2: 用户 Email Auth

**用户可见结果**：用户可通过 email OTP 登录，session 持久化 7 天

### 新建/修改文件

```
cozmio-api/src/
├── services/
│   ├── email.ts                 # Resend 发送邮件
│   └── session.ts               # OTP 生成/验证/存储
├── middleware/
│   └── require-auth.ts          # 验证用户 session
└── routes/
    ├── auth.ts                  # /api/auth/send-code, verify, logout, me
    └── index.ts                 # 路由注册
```

### src/services/session.ts

```typescript
import { v4 as uuidv4 } from "uuid";
import { eq } from "drizzle-orm";
import { db } from "../db/index.js";
import { sessions, users } from "../db/schema.js";

export async function sendOtp(email: string): Promise<string> {
  // 生成 6 位数字 OTP
  const code = Math.floor(100000 + Math.random() * 900000).toString();
  const expiresAt = new Date(Date.now() + 10 * 60 * 1000); // 10 min

  // 查找或创建 user
  let [user] = await db.select().from(users).where(eq(users.email, email)).limit(1);
  if (!user) {
    [user] = await db.insert(users).values({ email }).returning();
  }

  // 删除该 email 的旧 OTP sessions
  await db.delete(sessions).where(eq(sessions.email, email));

  // 创建新 OTP session
  await db.insert(sessions).values({
    userId: user.id,
    email,
    token: code,
    tokenType: "otp",
    expiresAt,
  });

  return code;
}

export async function verifyOtp(email: string, code: string) {
  const [session] = await db
    .select()
    .from(sessions)
    .where(eq(sessions.email, email))
    .limit(1);

  if (!session || session.token !== code || session.tokenType !== "otp" || session.expiresAt < new Date()) {
    throw ERRORS.UNAUTHORIZED("Invalid or expired verification code");
  }

  // 删除一次性 OTP session
  await db.delete(sessions).where(eq(sessions.id, session.id));

  // 创建持久 session（7 天）
  const sessionToken = uuidv4();
  const expiresAt = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000);

  await db.insert(sessions).values({
    userId: session.userId,
    email,
    token: sessionToken,
    tokenType: "session",
    expiresAt,
  });

  const [user] = await db.select().from(users).where(eq(users.id, session.userId!)).limit(1);
  return { user, sessionToken, expiresAt };
}

export async function validateSession(token: string) {
  const [session] = await db
    .select()
    .from(sessions)
    .where(eq(sessions.token, token))
    .limit(1);

  if (!session || session.tokenType !== "session" || session.expiresAt < new Date()) {
    return null;
  }

  const [user] = await db.select().from(users).where(eq(users.id, session.userId!)).limit(1);
  return user || null;
}

export async function destroySession(token: string) {
  await db.delete(sessions).where(eq(sessions.token, token));
}
```

### src/services/email.ts

```typescript
import { Resend } from "resend";

const resend = new Resend(process.env.RESEND_API_KEY);

export async function sendOtpEmail(email: string, code: string) {
  await resend.emails.send({
    from: "Cozmio <noreply@cozmio.net>",
    to: email,
    subject: "Your Cozmio verification code",
    html: `
      <div style="font-family: sans-serif; max-width: 480px; margin: 0 auto;">
        <h2 style="color: #151515;">Your verification code</h2>
        <p style="font-size: 24px; letter-spacing: 4px; font-weight: bold;">${code}</p>
        <p style="color: #625b54;">This code expires in 10 minutes. If you didn't request this, please ignore this email.</p>
      </div>
    `,
  });
}
```

### src/middleware/require-auth.ts

```typescript
import { FastifyRequest, FastifyReply } from "fastify";
import { validateSession } from "../services/session.js";

declare module "fastify" {
  interface FastifyRequest {
    user?: Awaited<ReturnType<typeof validateSession>>;
  }
}

export async function requireAuth(request: FastifyRequest, reply: FastifyReply) {
  const token = request.cookies.session_token || request.headers.authorization?.replace("Bearer ", "");
  if (!token) {
    return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Not authenticated" } });
  }

  const user = await validateSession(token);
  if (!user) {
    return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Session expired" } });
  }

  request.user = user;
}
```

### src/routes/auth.ts

```typescript
import { FastifyInstance } from "fastify";
import { sendOtp, verifyOtp, destroySession } from "../services/session.js";
import { sendOtpEmail } from "../services/email.js";
import { ok, error } from "../lib/response.js";
import { ERRORS } from "../lib/errors.js";

export async function authRoutes(app: FastifyInstance) {
  app.post("/send-code", async (request, reply) => {
    const { email, type } = request.body as { email: string; type: "otp" | "magic_link" };
    if (!email || !email.includes("@")) {
      return error("VALIDATION_ERROR", "Invalid email", 400);
    }

    // Rate limit: 同一邮箱 60 秒内不能重复发送（简单检查）
    // 实际实现应该在 Redis 或 DB 中记录上次发送时间

    const code = await sendOtp(email);
    if (process.env.NODE_ENV !== "test") {
      await sendOtpEmail(email, code);
    }

    return ok({ message: "Verification code sent" });
  });

  app.post("/verify", async (request, reply) => {
    const { email, token } = request.body as { email: string; token: string };
    if (!email || !token) {
      return error("VALIDATION_ERROR", "Email and token are required", 400);
    }

    const { user, sessionToken, expiresAt } = await verifyOtp(email, token);

    reply.setCookie("session_token", sessionToken, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      expires: expiresAt,
    });

    return ok({ user, session_token: sessionToken, expires_at: expiresAt });
  });

  app.post("/logout", async (request, reply) => {
    const token = request.cookies.session_token;
    if (token) {
      await destroySession(token);
    }
    reply.clearCookie("session_token", { path: "/" });
    return ok({ message: "Logged out" });
  });

  app.get("/me", { preHandler: [app.authenticate] }, async (request, reply) => {
    return ok(request.user!);
  });
}
```

注意：`app.authenticate` 是 require-auth 中间件在 authRoutes 上的注册方式，实际实现时需在 buildApp 或单独文件中注册中间件。

### 验证

```bash
# 发送验证码（查看日志中的 code）
curl -X POST http://localhost:3000/api/auth/send-code \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","type":"otp"}'

# 数据库中查询
psql $DATABASE_URL -c "SELECT token FROM sessions WHERE email='test@example.com'"

# 验证
curl -X POST http://localhost:3000/api/auth/verify \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","token":"123456"}' \
  -c cookies.txt
```

**状态**：已锁定 ✓

---

## Slice 3: User Tasks API

**用户可见结果**：登录用户可创建任务、查看任务列表、查看任务详情

### 涉及文件

```
cozmio-api/src/routes/tasks.ts
```

### src/routes/tasks.ts

```typescript
import { FastifyInstance } from "fastify";
import { eq, desc, and, or, sql } from "drizzle-orm";
import { db } from "../db/index.js";
import { tasks, taskEvents, sessions } from "../db/schema.js";
import { requireAuth } from "../middleware/require-auth.js";
import { ok, error, paginated } from "../lib/response.js";
import { ERRORS } from "../lib/errors.js";

export async function tasksRoutes(app: FastifyInstance) {
  // 临时方案：直接在路由中应用 requireAuth
  const authMiddleware = async (request: any, reply: any) => {
    const token = request.cookies.session_token;
    if (!token) return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED" } });
    const [session] = await db.select().from(sessions).where(eq(sessions.token, token)).limit(1);
    if (!session || session.expiresAt < new Date()) return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED" } });
    const [user] = await db.select().from(users).where(eq(users.id, session.userId!)).limit(1);
    if (!user) return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED" } });
    request.user = user;
  };

  // GET /api/tasks — 任务列表
  app.get("/", { preHandler: [authMiddleware] }, async (request: any, reply) => {
    const { page = "1", pageSize = "20", status } = request.query;
    const pageNum = Math.max(1, parseInt(page));
    const size = Math.min(100, Math.max(1, parseInt(pageSize)));
    const offset = (pageNum - 1) * size;

    const conditions = [eq(tasks.userId, request.user.id)];
    if (status) conditions.push(eq(tasks.status, status));

    const [countResult] = await db.select({ count: sql<number>`count(*)` }).from(tasks).where(and(...conditions));
    const total = Number(countResult?.count ?? 0);

    const rows = await db
      .select({ id: tasks.id, title: tasks.title, status: tasks.status, createdAt: tasks.createdAt, updatedAt: tasks.updatedAt })
      .from(tasks)
      .where(and(...conditions))
      .orderBy(desc(tasks.createdAt))
      .limit(size)
      .offset(offset);

    return paginated(rows, pageNum, size, total);
  });

  // POST /api/tasks — 创建任务
  app.post("/", { preHandler: [authMiddleware] }, async (request: any, reply) => {
    const { title, prompt, sourceUrl, sourceType } = request.body as any;
    if (!title || !prompt) {
      return error("VALIDATION_ERROR", "Title and prompt are required", 400);
    }

    const [task] = await db.insert(tasks).values({
      userId: request.user.id,
      email: request.user.email,
      title,
      prompt,
      sourceUrl: sourceUrl || null,
      sourceType: sourceType || "manual",
      status: "submitted",
    }).returning();

    await db.insert(taskEvents).values({
      taskId: task.id,
      eventType: "created",
      message: "Task submitted",
    });

    return ok({ id: task.id, status: task.status });
  });

  // GET /api/tasks/:id — 任务详情
  app.get("/:id", async (request: any, reply) => {
    const { id } = request.params;
    const token = request.cookies.session_token;

    const [task] = await db.select().from(tasks).where(eq(tasks.id, id)).limit(1);
    if (!task) return error("NOT_FOUND", "Task not found", 404);

    // 验证访问权限：user_id 匹配 或拥有 share_token
    if (task.userId) {
      if (token) {
        const [session] = await db.select().from(sessions).where(eq(sessions.token, token)).limit(1);
        if (session && session.userId === task.userId) {
          // 已授权
        } else {
          return error("FORBIDDEN", "Access denied", 403);
        }
      } else {
        return error("UNAUTHORIZED", "Not authenticated", 401);
      }
    }

    const events = await db.select().from(taskEvents).where(eq(taskEvents.taskId, id)).orderBy(taskEvents.createdAt);

    return ok({ ...task, events });
  });
}
```

### 验证

```bash
# 先登录获取 session_token cookie
curl -X POST http://localhost:3000/api/auth/verify \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","token":"上一节的验证码"}' \
  -c cookies.txt

# 创建任务
curl -X POST http://localhost:3000/api/tasks \
  -H "Content-Type: application/json" \
  -b cookies.txt \
  -d '{"title":"测试任务","prompt":"分析代码","source_type":"manual"}'

# 获取列表
curl http://localhost:3000/api/tasks -b cookies.txt

# 获取详情
curl http://localhost:3000/api/tasks/[返回的id] -b cookies.txt
```

**状态**：已锁定 ✓

---

## Slice 4: /use 前端页面

**用户可见结果**：完整 /use workspace，含 email 验证、任务提交、历史列表、详情页

### 新建文件

```
cozmio/web/src/
├── app/
│   ├── use/
│   │   ├── page.tsx
│   │   ├── layout.tsx
│   │   └── tasks/
│   │       ├── page.tsx
│   │       └── [id]/
│   │           └── page.tsx
├── lib/
│   ├── api.ts
│   └── types.ts
└── components/
    └── use/
        ├── EmailLoginForm.tsx
        ├── Workspace.tsx
        ├── TaskSubmitForm.tsx
        ├── TaskList.tsx
        ├── TaskCard.tsx
        ├── TaskDetail.tsx
        └── TaskStatusBadge.tsx
```

### src/lib/types.ts

```typescript
export interface User {
  id: string;
  email: string;
  name?: string;
  role: "user" | "admin";
}

export interface Task {
  id: string;
  userId?: string;
  email: string;
  title: string;
  prompt: string;
  sourceUrl?: string;
  sourceType: string;
  status: TaskStatus;
  resultSummary?: string;
  resultPayload?: any;
  errorMessage?: string;
  internalNote?: string;
  shareToken?: string;
  createdAt: string;
  updatedAt: string;
  events?: TaskEvent[];
}

export type TaskStatus = "submitted" | "queued" | "processing" | "needs_review" | "done" | "failed" | "cancelled";

export interface TaskEvent {
  id: string;
  taskId: string;
  eventType: "created" | "status_change" | "note_added" | "email_sent" | "result_added";
  message?: string;
  metadata?: any;
  createdAt: string;
}

export interface ApiResponse<T> {
  ok: boolean;
  data?: T;
  error?: { code: string; message: string };
  pagination?: { page: number; pageSize: number; total: number };
}
```

### src/lib/api.ts

```typescript
const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

async function fetchApi<T>(path: string, options: RequestInit = {}): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    ...options,
    headers: {
      "Content-Type": "application/json",
      ...options.headers,
    },
    credentials: "include",
  });

  const json: ApiResponse<T> = await res.json();

  if (!json.ok) {
    throw new Error(json.error?.message || "API Error");
  }

  return json.data as T;
}

// Auth
export const api = {
  auth: {
    sendCode: (email: string) => fetchApi("/api/auth/send-code", { method: "POST", body: JSON.stringify({ email, type: "otp" }) }),
    verify: (email: string, token: string) => fetchApi<{ user: User; session_token: string; expires_at: string }>("/api/auth/verify", { method: "POST", body: JSON.stringify({ email, token }) }),
    logout: () => fetchApi("/api/auth/logout", { method: "POST" }),
    me: () => fetchApi<User>("/api/auth/me"),
  },
  tasks: {
    create: (data: { title: string; prompt: string; source_url?: string; source_type?: string }) =>
      fetchApi<{ id: string; status: string }>("/api/tasks", { method: "POST", body: JSON.stringify(data) }),
    list: (params?: { page?: number; pageSize?: number; status?: string }) => {
      const qs = new URLSearchParams(params as any).toString();
      return fetchApi<{ tasks: Task[] } & { pagination: any }>(`/api/tasks${qs ? `?${qs}` : ""}`);
    },
    get: (id: string) => fetchApi<Task>(`/api/tasks/${id}`),
  },
};
```

### src/components/use/TaskStatusBadge.tsx

```typescript
import { TaskStatus } from "@/lib/types";

const STATUS_CONFIG: Record<TaskStatus, { label: string; color: string }> = {
  submitted: { label: "已提交", color: "bg-gray-100 text-gray-600" },
  queued: { label: "排队中", color: "bg-blue-100 text-blue-700" },
  processing: { label: "处理中", color: "bg-orange-100 text-orange-700" },
  needs_review: { label: "待审核", color: "bg-purple-100 text-purple-700" },
  done: { label: "已完成", color: "bg-green-100 text-green-700" },
  failed: { label: "失败", color: "bg-red-100 text-red-700" },
  cancelled: { label: "已取消", color: "bg-gray-100 text-gray-500" },
};

export function TaskStatusBadge({ status }: { status: TaskStatus }) {
  const config = STATUS_CONFIG[status] || STATUS_CONFIG.submitted;
  return (
    <span className={`inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-semibold ${config.color}`}>
      {config.label}
    </span>
  );
}
```

### src/app/use/page.tsx

```typescript
"use client";
import { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { User } from "@/lib/types";
import { EmailLoginForm } from "@/components/use/EmailLoginForm";
import { Workspace } from "@/components/use/Workspace";

export default function UsePage() {
  const [user, setUser] = useState<User | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.auth.me()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="flex min-h-screen items-center justify-center">
        <div className="h-8 w-8 animate-spin rounded-full border-4 border-gray-200 border-t-[#151515]" />
      </div>
    );
  }

  if (!user) {
    return <EmailLoginForm onSuccess={(u) => setUser(u)} />;
  }

  return <Workspace user={user} onLogout={() => setUser(null)} />;
}
```

### src/components/use/EmailLoginForm.tsx

```typescript
"use client";
import { useState } from "react";
import { api } from "@/lib/api";
import { User } from "@/lib/types";

export function EmailLoginForm({ onSuccess }: { onSuccess: (user: User) => void }) {
  const [step, setStep] = useState<"email" | "code">("email");
  const [email, setEmail] = useState("");
  const [code, setCode] = useState("");
  const [countdown, setCountdown] = useState(0);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");

  async function handleSendCode() {
    if (!email || !email.includes("@")) {
      setError("请输入有效的邮箱地址");
      return;
    }
    setLoading(true);
    setError("");
    try {
      await api.auth.sendCode(email);
      setStep("code");
      setCountdown(60);
      const timer = setInterval(() => {
        setCountdown((c) => {
          if (c <= 1) { clearInterval(timer); return 0; }
          return c - 1;
        });
      }, 1000);
    } catch (e: any) {
      setError(e.message || "发送失败，请稍后重试");
    } finally {
      setLoading(false);
    }
  }

  async function handleVerify() {
    setLoading(true);
    setError("");
    try {
      const data = await api.auth.verify(email, code);
      onSuccess(data.user);
    } catch (e: any) {
      setError(e.message || "验证码错误或已过期");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center bg-[#faf8f5]">
      <div className="w-full max-w-md rounded-3xl bg-white p-10 shadow-xl">
        <h1 className="text-3xl font-bold text-[#151515]">进入 Cozmio Use</h1>
        <p className="mt-2 text-[#625b54]">输入邮箱开始使用</p>

        {step === "email" ? (
          <div className="mt-8 space-y-4">
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="your@email.com"
              className="coz-input w-full"
            />
            <button
              onClick={handleSendCode}
              disabled={loading}
              className="coz-btn-dark w-full"
            >
              {loading ? "发送中..." : "发送验证码"}
            </button>
          </div>
        ) : (
          <div className="mt-8 space-y-4">
            <p className="text-sm text-[#625b54]">
              验证码已发送到 <span className="font-bold">{email}</span>
            </p>
            <input
              type="text"
              value={code}
              onChange={(e) => setCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
              placeholder="6位验证码"
              className="coz-input w-full text-center text-2xl tracking-widest"
              maxLength={6}
            />
            <button
              onClick={handleVerify}
              disabled={loading || code.length !== 6}
              className="coz-btn-dark w-full"
            >
              {loading ? "验证中..." : "验证并进入"}
            </button>
            <button
              onClick={() => {
                setCountdown(60);
                handleSendCode();
              }}
              disabled={countdown > 0}
              className="w-full text-sm text-[#625b54] underline"
            >
              {countdown > 0 ? `${countdown}秒后可重新发送` : "重新发送验证码"}
            </button>
          </div>
        )}

        {error && (
          <div className="mt-4 rounded-xl bg-red-50 px-4 py-3 text-sm text-red-600">
            {error}
          </div>
        )}
      </div>
    </div>
  );
}
```

### src/components/use/Workspace.tsx

```typescript
"use client";
import { useState } from "react";
import { api } from "@/lib/api";
import { User, Task } from "@/lib/types";
import { TaskSubmitForm } from "./TaskSubmitForm";
import { TaskList } from "./TaskList";
import { TaskDetail } from "./TaskDetail";
import { ChevronDown, LogOut } from "lucide-react";

type Tab = "submit" | "history";

export function Workspace({ user, onLogout }: { user: User; onLogout: () => void }) {
  const [tab, setTab] = useState<Tab>("submit");
  const [selectedTaskId, setSelectedTaskId] = useState<string | null>(null);

  if (selectedTaskId) {
    return (
      <div className="min-h-screen bg-[#faf8f5]">
        <header className="flex items-center justify-between border-b border-black/6 bg-white px-8 py-4">
          <div className="flex items-center gap-4">
            <span className="font-bold text-[#151515]">Cozmio Use</span>
            <button onClick={() => setSelectedTaskId(null)} className="text-sm text-[#625b54]">
              ← 返回
            </button>
          </div>
        </header>
        <div className="mx-auto max-w-3xl px-8 py-8">
          <TaskDetail taskId={selectedTaskId} />
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-[#faf8f5]">
      <header className="flex items-center justify-between border-b border-black/6 bg-white px-8 py-4">
        <div className="flex items-center gap-4">
          <span className="font-bold text-[#151515]">Cozmio Use</span>
          <nav className="flex gap-1">
            {(["submit", "history"] as const).map((t) => (
              <button
                key={t}
                onClick={() => setTab(t)}
                className={`rounded-xl px-5 py-2 text-sm font-bold transition-colors ${
                  tab === t ? "bg-[#151515] text-white" : "text-[#625b54] hover:bg-[#f0ece6]"
                }`}
              >
                {t === "submit" ? "提交任务" : "任务历史"}
              </button>
            ))}
          </nav>
        </div>
        <div className="flex items-center gap-3">
          <span className="text-sm text-[#625b54]">{user.email}</span>
          <button
            onClick={async () => { await api.auth.logout(); onLogout(); }}
            className="flex items-center gap-2 rounded-xl px-4 py-2 text-sm font-bold text-[#625b54] hover:bg-[#f0ece6]"
          >
            <LogOut className="h-4 w-4" /> 退出
          </button>
        </div>
      </header>

      <div className="mx-auto max-w-3xl px-8 py-8">
        {tab === "submit" ? (
          <TaskSubmitForm onSuccess={() => setTab("history")} />
        ) : (
          <TaskList onTaskClick={setSelectedTaskId} />
        )}
      </div>
    </div>
  );
}
```

### src/components/use/TaskSubmitForm.tsx

```typescript
"use client";
import { useState } from "react";
import { api } from "@/lib/api";
import { ArrowRight, CheckCircle2 } from "lucide-react";

export function TaskSubmitForm({ onSuccess }: { onSuccess?: () => void }) {
  const [title, setTitle] = useState("");
  const [prompt, setPrompt] = useState("");
  const [sourceUrl, setSourceUrl] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState("");

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    if (!title.trim() || !prompt.trim()) {
      setError("请填写标题和任务描述");
      return;
    }
    setLoading(true);
    setError("");
    setSuccess("");
    try {
      const data = await api.tasks.create({ title, prompt, source_url: sourceUrl, source_type: "manual" });
      setSuccess(`任务已提交，ID: ${data.id}`);
      setTitle("");
      setPrompt("");
      setSourceUrl("");
      setTimeout(() => setSuccess(""), 5000);
      onSuccess?.();
    } catch (e: any) {
      setError(e.message || "提交失败，请稍后重试");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="rounded-3xl bg-white p-8 shadow-sm">
      <h2 className="text-2xl font-bold">提交新任务</h2>
      <form onSubmit={handleSubmit} className="mt-6 space-y-5">
        <div>
          <label className="mb-2 block text-sm font-bold">任务标题 *</label>
          <input
            value={title}
            onChange={(e) => setTitle(e.target.value.slice(0, 200))}
            className="coz-input w-full"
            placeholder="简短描述你要完成的任务"
            maxLength={200}
          />
          <div className="mt-1 text-right text-xs text-[#aaa098]">{title.length}/200</div>
        </div>
        <div>
          <label className="mb-2 block text-sm font-bold">任务描述 *</label>
          <textarea
            value={prompt}
            onChange={(e) => setPrompt(e.target.value.slice(0, 5000))}
            className="coz-input w-full resize-none"
            rows={6}
            placeholder="详细描述任务目标、期望的成果、背景信息..."
            maxLength={5000}
          />
          <div className="mt-1 text-right text-xs text-[#aaa098]">{prompt.length}/5000</div>
        </div>
        <div>
          <label className="mb-2 block text-sm font-bold">相关链接（可选）</label>
          <input
            value={sourceUrl}
            onChange={(e) => setSourceUrl(e.target.value)}
            className="coz-input w-full"
            placeholder="https://..."
          />
        </div>
        <button type="submit" disabled={loading} className="coz-btn-dark flex items-center gap-2">
          {loading ? "提交中..." : "提交任务"} <ArrowRight className="h-4 w-4" />
        </button>
        {error && <div className="rounded-xl bg-red-50 px-4 py-3 text-sm text-red-600">{error}</div>}
        {success && (
          <div className="flex items-center gap-2 rounded-xl bg-green-50 px-4 py-3 text-sm text-green-700">
            <CheckCircle2 className="h-4 w-4" /> {success}
          </div>
        )}
      </form>
    </div>
  );
}
```

### src/components/use/TaskList.tsx

```typescript
"use client";
import { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Task } from "@/lib/types";
import { TaskCard } from "./TaskCard";
import { InboxIcon } from "lucide-react";

export function TaskList({ onTaskClick }: { onTaskClick: (id: string) => void }) {
  const [tasks, setTasks] = useState<Task[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    api.tasks.list()
      .then((res) => setTasks(res.data || []))
      .catch((e: any) => setError(e.message))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return (
      <div className="space-y-4">
        {[1, 2, 3].map((i) => (
          <div key={i} className="h-24 animate-pulse rounded-2xl bg-gray-100" />
        ))}
      </div>
    );
  }

  if (error) {
    return (
      <div className="rounded-2xl bg-red-50 p-6 text-center text-red-600">
        <p>{error}</p>
        <button onClick={() => window.location.reload()} className="mt-2 underline">重试</button>
      </div>
    );
  }

  if (tasks.length === 0) {
    return (
      <div className="rounded-3xl bg-white p-16 text-center shadow-sm">
        <InboxIcon className="mx-auto h-16 w-16 text-[#cbc5ff]" />
        <h3 className="mt-4 text-xl font-bold">还没有任务</h3>
        <p className="mt-2 text-[#625b54]">提交你的第一个任务，开启高效协作</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {tasks.map((task) => (
        <TaskCard key={task.id} task={task} onClick={() => onTaskClick(task.id)} />
      ))}
    </div>
  );
}
```

### src/components/use/TaskCard.tsx

```typescript
import { Task } from "@/lib/types";
import { TaskStatusBadge } from "./TaskStatusBadge";
import { formatDistanceToNow } from "@/lib/utils";

export function TaskCard({ task, onClick }: { task: Task; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="w-full rounded-2xl bg-white p-6 text-left shadow-sm transition-shadow hover:shadow-md"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="min-w-0 flex-1">
          <h3 className="truncate font-bold text-[#151515]">{task.title}</h3>
          {task.resultSummary && (
            <p className="mt-2 line-clamp-2 text-sm text-[#625b54]">{task.resultSummary}</p>
          )}
          <p className="mt-2 text-xs text-[#aaa098]">
            {formatDistanceToNow(new Date(task.createdAt))}
          </p>
        </div>
        <TaskStatusBadge status={task.status} />
      </div>
    </button>
  );
}
```

### src/components/use/TaskDetail.tsx

```typescript
"use client";
import { useState, useEffect } from "react";
import { api } from "@/lib/api";
import { Task } from "@/lib/types";
import { TaskStatusBadge } from "./TaskStatusBadge";

export function TaskDetail({ taskId }: { taskId: string }) {
  const [task, setTask] = useState<Task | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState("");

  useEffect(() => {
    api.tasks.get(taskId)
      .then(setTask)
      .catch((e: any) => setError(e.message))
      .finally(() => setLoading(false));
  }, [taskId]);

  if (loading) return <div className="h-64 animate-pulse rounded-2xl bg-gray-100" />;
  if (error) return <div className="rounded-2xl bg-red-50 p-6 text-red-600">{error}</div>;
  if (!task) return null;

  return (
    <div className="space-y-6 rounded-3xl bg-white p-8 shadow-sm">
      <div className="flex items-start justify-between gap-4">
        <h1 className="text-3xl font-bold text-[#151515]">{task.title}</h1>
        <TaskStatusBadge status={task.status} />
      </div>
      <div className="text-sm text-[#aaa098]">
        提交于 {new Date(task.createdAt).toLocaleString("zh-CN")}
        {task.updatedAt !== task.createdAt && (
          <> · 更新于 {new Date(task.updatedAt).toLocaleString("zh-CN")}</>
        )}
      </div>
      <div>
        <h2 className="mb-2 font-bold">任务描述</h2>
        <pre className="whitespace-pre-wrap rounded-xl bg-[#faf8f5] p-4 text-sm">{task.prompt}</pre>
      </div>
      {task.resultSummary && (
        <div>
          <h2 className="mb-2 font-bold">执行结果</h2>
          <div className="rounded-xl bg-green-50 p-4 text-sm">{task.resultSummary}</div>
        </div>
      )}
      {task.errorMessage && (
        <div>
          <h2 className="mb-2 font-bold">错误信息</h2>
          <div className="rounded-xl bg-red-50 p-4 text-sm text-red-700">{task.errorMessage}</div>
        </div>
      )}
    </div>
  );
}
```

### 验证

```bash
cd web
NEXT_PUBLIC_API_BASE_URL=http://localhost:3000 npm run dev
# 浏览器打开 http://localhost:3000/use
# 1. 输入 email → 点发送验证码
# 2. 查看后端日志或数据库获取验证码
# 3. 输入验证码 → 进入 workspace
# 4. 提交任务 → 显示成功
# 5. 点击任务历史 → 显示任务列表
# 6. 点击任务卡片 → 查看任务详情
```

**状态**：已锁定 ✓

---

## Slice 5: Admin Auth + Middleware

**用户可见结果**：Admin 可登录 /admin，session 持久化 24 小时

### 新建文件

```
cozmio-api/src/
├── middleware/
│   └── require-admin.ts
├── services/
│   └── admin-session.ts
├── routes/admin/
│   ├── auth.ts
│   └── index.ts
```

### src/middleware/require-admin.ts

```typescript
import { FastifyRequest, FastifyReply } from "fastify";
import { eq } from "drizzle-orm";
import { db } from "../db/index.js";
import { adminSessions } from "../db/schema.js";
import { AppError, ERRORS } from "../lib/errors.js";

export async function requireAdmin(request: FastifyRequest, reply: FastifyReply) {
  const token = request.cookies.admin_session || request.headers.authorization?.replace("Bearer ", "");
  if (!token) {
    throw ERRORS.UNAUTHORIZED("Admin not authenticated");
  }

  const [session] = await db
    .select()
    .from(adminSessions)
    .where(eq(adminSessions.token, token))
    .limit(1);

  if (!session || session.expiresAt < new Date()) {
    throw ERRORS.UNAUTHORIZED("Admin session expired");
  }

  if (session.email !== process.env.ADMIN_EMAIL) {
    throw ERRORS.FORBIDDEN("Invalid admin");
  }
}
```

### src/services/admin-session.ts

```typescript
import { v4 as uuidv4 } from "uuid";
import { eq } from "drizzle-orm";
import { db } from "../db/index.js";
import { adminSessions } from "../db/schema.js";

export async function createAdminSession(email: string) {
  const token = uuidv4();
  const expiresAt = new Date(Date.now() + 24 * 60 * 60 * 1000); // 24 hours
  await db.insert(adminSessions).values({ email, token, expiresAt });
  return { token, expiresAt };
}

export async function destroyAdminSession(token: string) {
  await db.delete(adminSessions).where(eq(adminSessions.token, token));
}

export async function validateAdminSession(token: string) {
  const [session] = await db.select().from(adminSessions).where(eq(adminSessions.token, token)).limit(1);
  if (!session || session.expiresAt < new Date()) return null;
  return session;
}
```

### src/routes/admin/auth.ts

```typescript
import { FastifyInstance } from "fastify";
import bcrypt from "bcrypt";
import { createAdminSession, destroyAdminSession, validateAdminSession } from "../../services/admin-session.js";
import { requireAdmin } from "../../middleware/require-admin.js";
import { ok, error } from "../../lib/response.js";
import { ERRORS } from "../../lib/errors.js";

export async function adminAuthRoutes(app: FastifyInstance) {
  app.post("/login", async (request, reply) => {
    const { email, password } = request.body as any;

    if (email !== process.env.ADMIN_EMAIL) {
      return error("UNAUTHORIZED", "Invalid credentials", 401);
    }

    const valid = await bcrypt.compare(password, process.env.ADMIN_PASSWORD_HASH!);
    if (!valid) {
      return error("UNAUTHORIZED", "Invalid credentials", 401);
    }

    const { token, expiresAt } = await createAdminSession(email);

    reply.setCookie("admin_session", token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      expires: expiresAt,
    });

    return ok({ session_token: token, expires_at: expiresAt });
  });

  app.post("/logout", { preHandler: [requireAdmin] }, async (request, reply) => {
    const token = request.cookies.admin_session;
    if (token) {
      await destroyAdminSession(token);
    }
    reply.clearCookie("admin_session", { path: "/" });
    return ok({ message: "Logged out" });
  });

  app.get("/me", { preHandler: [requireAdmin] }, async (request, reply) => {
    return ok({ email: process.env.ADMIN_EMAIL });
  });
}
```

### 验证

```bash
# 生成 admin password hash
node -e "const bcrypt = require('bcrypt'); bcrypt.hash('adminpassword', 10).then(h => console.log(h))"
# 将结果填入 ADMIN_PASSWORD_HASH

# 登录
curl -X POST http://localhost:3000/api/admin/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@cozmio.net","password":"adminpassword"}' \
  -c admin_cookies.txt

# 获取 admin 信息
curl http://localhost:3000/api/admin/me -b admin_cookies.txt

# 登出
curl -X POST http://localhost:3000/api/admin/logout -b admin_cookies.txt
```

**状态**：已锁定 ✓

---

## Slice 6: Admin CRUD APIs

**用户可见结果**：所有 admin CRUD API 可用

### src/routes/admin/applications.ts

```typescript
import { FastifyInstance } from "fastify";
import { eq, like, desc, sql } from "drizzle-orm";
import { db } from "../../db/index.js";
import { applications } from "../../db/schema.js";
import { requireAdmin } from "../../middleware/require-admin.js";
import { ok, error, paginated } from "../../lib/response.js";

export async function adminApplicationsRoutes(app: FastifyInstance) {
  app.get("/applications", { preHandler: [requireAdmin] }, async (request) => {
    const { page = "1", pageSize = "20", status, search } = request.query as any;
    const pageNum = Math.max(1, parseInt(page));
    const size = Math.min(100, Math.max(1, parseInt(pageSize)));
    const offset = (pageNum - 1) * size;

    const conditions: any[] = [];
    if (status) conditions.push(eq(applications.status, status));
    if (search) {
      conditions.push(
        sql`(${applications.name} ILIKE ${"%" + search + "%"} OR ${applications.email} ILIKE ${"%" + search + "%"} OR ${applications.company} ILIKE ${"%" + search + "%"})`
      );
    }

    const where = conditions.length > 0 ? sql.join(conditions, sql` AND `) : sql`1=1`;

    const [countResult] = await db.select({ count: sql<number>`count(*)` }).from(applications).where(sql`${where}`));
    const total = Number(countResult?.count ?? 0);

    const rows = await db
      .select()
      .from(applications)
      .where(sql`${where}`)
      .orderBy(desc(applications.createdAt))
      .limit(size)
      .offset(offset);

    return paginated(rows, pageNum, size, total);
  });

  app.get("/applications/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params;
    const [app] = await db.select().from(applications).where(eq(applications.id, id)).limit(1);
    if (!app) return error("NOT_FOUND", "Application not found", 404);
    return ok(app);
  });

  app.patch("/applications/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params;
    const { status, adminNote } = request.body as any;
    const [updated] = await db
      .update(applications)
      .set({ status, adminNote, updatedAt: new Date() })
      .where(eq(applications.id, id))
      .returning();
    if (!updated) return error("NOT_FOUND", "Application not found", 404);
    return ok(updated);
  });
}
```

### src/routes/admin/tasks.ts

类似 applications，实现 GET/PATCH /api/admin/tasks 和 /:id，含 task_events 联查。

### src/routes/admin/users.ts

实现 GET /api/admin/users 和 /:id，PATCH /:id 修改 access flags。

### src/routes/admin/downloads.ts

实现 GET /api/admin/downloads, POST / (上传文件到 R2，保存 record), PATCH /:id。

### src/routes/admin/index.ts

```typescript
import { FastifyInstance } from "fastify";
import { adminAuthRoutes } from "./auth.js";
import { adminApplicationsRoutes } from "./applications.js";
import { adminTasksRoutes } from "./tasks.js";
import { adminUsersRoutes } from "./users.js";
import { adminDownloadsRoutes } from "./downloads.js";

export async function adminRoutes(app: FastifyInstance) {
  app.register(adminAuthRoutes);
  app.register(adminApplicationsRoutes);
  app.register(adminTasksRoutes);
  app.register(adminUsersRoutes);
  app.register(adminDownloadsRoutes);
}
```

**验证**：每个 API 逐一测试（见设计文档 Section 11.1）

**状态**：已锁定 ✓

---

## Slice 7: /admin 前端页面

**用户可见结果**：完整 admin 后台，所有模块可操作

### 新建文件

```
cozmio/web/src/
├── app/
│   └── admin/
│       ├── page.tsx          # 登录页
│       ├── layout.tsx        # admin layout（侧边栏）
│       ├── dashboard/
│       │   └── page.tsx
│       ├── applications/
│       │   └── page.tsx
│       ├── tasks/
│       │   └── page.tsx
│       ├── users/
│       │   └── page.tsx
│       └── downloads/
│           └── page.tsx
└── components/
    └── admin/
        ├── AdminSidebar.tsx
        ├── AdminTable.tsx
        ├── StatusBadge.tsx
        ├── DetailModal.tsx
        └── Toast.tsx
```

### 页面结构

**/admin/page.tsx（登录页）**：
- 如果已有 admin session → 重定向 /admin/dashboard
- 未登录：显示 email + 密码表单
- 登录失败：显示错误信息

**/admin/layout.tsx**：
- 左侧 Sidebar：Dashboard / Applications / Tasks / Users / Downloads
- 右上：Admin email + Logout
- 内容区：`<Outlet />`

**/admin/dashboard/page.tsx**：
- 4 个统计卡片（User Count, Application Count, Task Count, Download Version Count）
- 最近 5 个新申请（表格）
- 最近 5 个新任务（表格）

**/admin/applications/page.tsx**：
- 搜索框（name/email/company）
- Status tabs: 全部 / new / reviewed / invited / rejected
- 表格：ID / Name / Email / Company / Status / Date / Actions
- Actions: 查看详情（弹窗）
- 详情弹窗：所有字段 + status 下拉 + admin_note 文本框 + 保存按钮

**/admin/tasks/page.tsx**：
- 类似 applications，含 status/result_summary/internal_note 修改

**/admin/users/page.tsx**：
- 用户列表 + access flags 修改（beta/desktop/hardware toggle）+ invite_code

**/admin/downloads/page.tsx**：
- 版本列表 + 添加/编辑弹窗（version/platform/access_level/changelog/file上传/is_latest/is_active）

### 验证

```bash
# 1. /admin/login → 输入 admin credentials → 进入 dashboard
# 2. Dashboard 显示统计
# 3. Applications 列表 + 筛选 + 详情弹窗修改 status
# 4. Tasks 列表 + 修改 status/result_summary/internal_note
# 5. Users 列表 + 修改 access flags
# 6. Downloads 列表 + 添加新版本
```

**状态**：已锁定 ✓

---

## Slice 8: Public APIs + 集成

**用户可见结果**：/request 表单提交成功，下载链接可用，/use 页面正确调用本后端

### 改动文件

```
cozmio/web/src/
├── lib/
│   ├── request-submit.ts     # 改造：指向本后端 URL
│   └── site-config.ts        # 改造：下载链接改为 /api/downloads/latest
```

### src/lib/request-submit.ts（改造）

```typescript
const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "http://localhost:3000";

export async function submitCozmioRequest(payload: CozmioRequestPayload) {
  const response = await fetch(`${API_BASE}/api/applications`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  const result = await response.json().catch(() => ({}));
  if (!response.ok || result.ok === false) {
    throw new Error(result.error?.message || "提交失败，请稍后再试");
  }
  return result;
}
```

### src/lib/site-config.ts（改造下载链接）

```typescript
// downloads.windows 改为调用 API 获取
// 页面中使用 fetch 动态获取，不硬编码 URL
export async function getLatestDownloadUrl(platform: string) {
  const res = await fetch(`${process.env.NEXT_PUBLIC_API_BASE_URL}/api/downloads/latest?platform=${platform}`);
  const json = await res.json();
  return json.ok ? json.data.download_url : null;
}
```

### cozmio-api/src/routes/downloads.ts

```typescript
import { FastifyInstance } from "fastify";
import { eq, and } from "drizzle-orm";
import { db } from "../db/index.js";
import { downloadVersions } from "../db/schema.js";
import { ok, error } from "../lib/response.js";
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";

const s3 = new S3Client({
  region: "auto",
  endpoint: `https://${process.env.R2_ACCOUNT_ID}.r2.cloudflarestorage.com`,
  credentials: {
    accessKeyId: process.env.R2_ACCESS_KEY_ID!,
    secretAccessKey: process.env.R2_SECRET_ACCESS_KEY!,
  },
});

export async function downloadsRoutes(app: FastifyInstance) {
  app.get("/latest", async (request, reply) => {
    const { platform = "windows" } = request.query;
    const [version] = await db
      .select()
      .from(downloadVersions)
      .where(and(eq(downloadVersions.platform, platform), eq(downloadVersions.isLatest, true), eq(downloadVersions.isActive, true)))
      .limit(1);

    if (!version) return error("NOT_FOUND", "No download available", 404);

    // 生成 R2 签名 URL（15 分钟有效）
    const command = new GetObjectCommand({
      Bucket: process.env.R2_BUCKET!,
      Key: version.fileKey,
    });
    const downloadUrl = await getSignedUrl(s3, command, { expiresIn: 15 * 60 });

    return ok({
      version: version.version,
      platform: version.platform,
      download_url: downloadUrl,
      changelog: version.changelog,
      access_level: version.accessLevel,
    });
  });
}
```

### seeds/seed.ts

```typescript
import { db } from "../src/db/index.js";
import { users, downloadVersions } from "../src/db/schema.js";
import { nanoid } from "nanoid";

async function seed() {
  // 创建 admin user（可选）
  const [admin] = await db.insert(users).values({
    email: "admin@cozmio.net",
    role: "admin",
    betaAccess: true,
    desktopAccess: true,
    hardwareAccess: true,
    inviteCode: nanoid(8).toUpperCase(),
  }).onConflictDoNothing().returning();

  // 创建初始下载版本
  await db.insert(downloadVersions).values({
    version: "1.0.0",
    platform: "windows",
    fileKey: "releases/cozmio-1.0.0-windows.exe",
    changelog: "Initial release",
    accessLevel: "public",
    isLatest: true,
    isActive: true,
  }).onConflictDoNothing();

  console.log("Seed completed");
  process.exit(0);
}

seed();
```

### 验证

```bash
# 1. 提交内测申请
curl -X POST http://localhost:3000/api/applications \
  -H "Content-Type: application/json" \
  -d '{"name":"张三","email":"zhang@example.com","company":"某公司","use_case":"测试","source":"web"}'

# 2. 前端 /request 页面提交
# 浏览器打开 http://localhost:3000/request
# 填写表单 → 提交 → 显示成功

# 3. 下载链接
curl "http://localhost:3000/api/downloads/latest?platform=windows"
```

**状态**：已锁定 ✓

---

## 实施优先级总结

| Slice | 文件数 | 依赖 | 风险 |
|-------|--------|------|------|
| 1. API 脚手架 | 12 | 无 | 低 |
| 2. User Auth | 6 | Slice 1 | 低 |
| 3. User Tasks API | 1 | Slice 2 | 低 |
| 4. /use 前端 | 12 | Slice 3 | 中（前端集成）|
| 5. Admin Auth | 4 | Slice 1 | 低 |
| 6. Admin CRUD APIs | 5 | Slice 5 | 低 |
| 7. /admin 前端 | 10 | Slice 6 | 中（前端集成）|
| 8. Public APIs + 集成 | 4 | Slice 4 | 低 |

**建议执行方式**：调用 `subagent-driven-development` 技能并行执行互相独立的 slices。