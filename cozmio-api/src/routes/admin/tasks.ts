import { FastifyInstance } from "fastify";
import { eq, desc, sql } from "drizzle-orm";
import { db } from "../../db/index.js";
import { tasks, taskEvents } from "../../db/schema.js";
import { requireAdmin } from "../../middleware/require-admin.js";
import { ok, error, paginated } from "../../lib/response.js";

export async function adminTasksRoutes(app: FastifyInstance) {
  // GET /api/admin/tasks
  app.get("/tasks", { preHandler: [requireAdmin] }, async (request) => {
    const { page = "1", pageSize = "20", status, search } = request.query as any;
    const pageNum = Math.max(1, parseInt(page));
    const size = Math.min(100, Math.max(1, parseInt(pageSize)));
    const offset = (pageNum - 1) * size;

    const conditions: any[] = [];
    if (status) conditions.push(eq(tasks.status, status));
    if (search) {
      conditions.push(
        sql`(${tasks.title} ILIKE ${"%" + search + "%"} OR ${tasks.email} ILIKE ${"%" + search + "%"})`
      );
    }

    const where = conditions.length > 0 ? sql.join(conditions, sql` AND `) : sql`1=1`;

    const [countResult] = await db.select({ count: sql<number>`count(*)` }).from(tasks).where(sql`${where}`);
    const total = Number(countResult?.count ?? 0);

    const rows = await db
      .select()
      .from(tasks)
      .where(sql`${where}`)
      .orderBy(desc(tasks.createdAt))
      .limit(size)
      .offset(offset);

    return paginated(rows, pageNum, size, total);
  });

  // GET /api/admin/tasks/:id
  app.get("/tasks/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    const [task] = await db.select().from(tasks).where(eq(tasks.id, id)).limit(1);
    if (!task) return error("NOT_FOUND", "Task not found", 404);

    const events = await db.select().from(taskEvents).where(eq(taskEvents.taskId, id)).orderBy(taskEvents.createdAt);
    return ok({ ...task, events });
  });

  // PATCH /api/admin/tasks/:id
  app.patch("/tasks/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    const { status, resultSummary, resultPayload, errorMessage, internalNote } = request.body as any;

    const [updated] = await db
      .update(tasks)
      .set({ status, resultSummary, resultPayload, errorMessage, internalNote, updatedAt: new Date() })
      .where(eq(tasks.id, id))
      .returning();

    if (!updated) return error("NOT_FOUND", "Task not found", 404);
    return ok(updated);
  });
}