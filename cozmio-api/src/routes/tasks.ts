import { FastifyInstance } from "fastify";
import { eq, desc, and, sql } from "drizzle-orm";
import { db } from "../db/index.js";
import { tasks, taskEvents, sessions, users } from "../db/schema.js";
import { validateSession } from "../services/session.js";
import { ok, error, paginated } from "../lib/response.js";
import { nanoid } from "nanoid";

export async function tasksRoutes(app: FastifyInstance) {
  // Auth middleware inline
  async function authMiddleware(request: any, reply: any) {
    const token = request.cookies.session_token;
    if (!token) {
      return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Not authenticated" } });
    }
    const user = await validateSession(token);
    if (!user) {
      return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Session expired" } });
    }
    request.user = user;
  }

  // GET /api/tasks — 任务列表
  app.get("/", { preHandler: [authMiddleware] }, async (request: any, reply) => {
    const { page = "1", pageSize = "20", status } = request.query as any;
    const pageNum = Math.max(1, parseInt(String(page)));
    const size = Math.min(100, Math.max(1, parseInt(String(pageSize))));
    const offset = (pageNum - 1) * size;

    const conditions = [eq(tasks.userId, request.user.id)];
    if (status) conditions.push(eq(tasks.status, status as any));

    const [countResult] = await db.select({ count: sql<number>`count(*)` }).from(tasks).where(and(...conditions));
    const total = Number(countResult?.count ?? 0);

    const rows = await db
      .select({
        id: tasks.id,
        title: tasks.title,
        status: tasks.status,
        createdAt: tasks.createdAt,
        updatedAt: tasks.updatedAt,
        resultSummary: tasks.resultSummary,
        errorMessage: tasks.errorMessage,
        shareToken: tasks.shareToken,
      })
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

    const shareToken = nanoid(16);

    const [task] = await db.insert(tasks).values({
      userId: request.user.id,
      email: request.user.email,
      title,
      prompt,
      sourceUrl: sourceUrl || null,
      sourceType: sourceType || "manual",
      status: "submitted",
      shareToken,
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
    const { id } = request.params as { id: string };
    const token = request.cookies.session_token;

    const [task] = await db.select().from(tasks).where(eq(tasks.id, id)).limit(1);
    if (!task) return error("NOT_FOUND", "Task not found", 404);

    // 验证访问权限
    if (task.userId) {
      if (token) {
        const user = await validateSession(token);
        if (!user || user.id !== task.userId) {
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