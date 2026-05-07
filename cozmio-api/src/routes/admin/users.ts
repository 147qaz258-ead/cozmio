import { FastifyInstance } from "fastify";
import { eq, desc, sql } from "drizzle-orm";
import { db } from "../../db/index.js";
import { users } from "../../db/schema.js";
import { requireAdmin } from "../../middleware/require-admin.js";
import { ok, error, paginated } from "../../lib/response.js";

export async function adminUsersRoutes(app: FastifyInstance) {
  // GET /api/admin/users
  app.get("/users", { preHandler: [requireAdmin] }, async (request) => {
    const { page = "1", pageSize = "20", search } = request.query as any;
    const pageNum = Math.max(1, parseInt(page));
    const size = Math.min(100, Math.max(1, parseInt(pageSize)));
    const offset = (pageNum - 1) * size;

    let where = sql`1=1`;
    if (search) {
      where = sql`${users.email} ILIKE ${"%" + search + "%"} OR ${users.name} ILIKE ${"%" + search + "%"}`;
    }

    const [countResult] = await db.select({ count: sql<number>`count(*)` }).from(users).where(where);
    const total = Number(countResult?.count ?? 0);

    const rows = await db
      .select()
      .from(users)
      .where(where)
      .orderBy(desc(users.createdAt))
      .limit(size)
      .offset(offset);

    return paginated(rows, pageNum, size, total);
  });

  // GET /api/admin/users/:id
  app.get("/users/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    const [user] = await db.select().from(users).where(eq(users.id, id)).limit(1);
    if (!user) return error("NOT_FOUND", "User not found", 404);
    return ok(user);
  });

  // PATCH /api/admin/users/:id
  app.patch("/users/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
    const { id } = request.params as { id: string };
    const { name, role, webAccess, betaAccess, desktopAccess, hardwareAccess, inviteCode } = request.body as any;

    const [updated] = await db
      .update(users)
      .set({ name, role, webAccess, betaAccess, desktopAccess, hardwareAccess, inviteCode, updatedAt: new Date() })
      .where(eq(users.id, id))
      .returning();

    if (!updated) return error("NOT_FOUND", "User not found", 404);
    return ok(updated);
  });
}