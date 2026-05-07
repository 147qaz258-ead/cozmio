import { FastifyInstance } from "fastify";
import { eq, desc, sql } from "drizzle-orm";
import { db } from "../../db/index.js";
import { applications } from "../../db/schema.js";
import { requireAdmin } from "../../middleware/require-admin.js";
import { ok, error, paginated } from "../../lib/response.js";

export async function adminApplicationsRoutes(app: FastifyInstance) {
  // GET /api/admin/applications - list with search and status filter
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

    const [countResult] = await db.select({ count: sql<number>`count(*)` }).from(applications).where(sql`${where}`);
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

  // GET /api/admin/applications/:id
  app.get("/applications/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    const [app] = await db.select().from(applications).where(eq(applications.id, id)).limit(1);
    if (!app) return error("NOT_FOUND", "Application not found", 404);
    return ok(app);
  });

  // PATCH /api/admin/applications/:id
  app.patch("/applications/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
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