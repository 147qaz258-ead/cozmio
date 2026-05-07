import { eq, desc, sql } from "drizzle-orm";
import { db } from "../../db/index.js";
import { downloadVersions } from "../../db/schema.js";
import { requireAdmin } from "../../middleware/require-admin.js";
import { ok, error, paginated } from "../../lib/response.js";
export async function adminDownloadsRoutes(app) {
    // GET /api/admin/downloads
    app.get("/downloads", { preHandler: [requireAdmin] }, async (request) => {
        const { page = "1", pageSize = "20", platform } = request.query;
        const pageNum = Math.max(1, parseInt(page));
        const size = Math.min(100, Math.max(1, parseInt(pageSize)));
        const offset = (pageNum - 1) * size;
        let where = sql `1=1`;
        if (platform)
            where = sql `${downloadVersions.platform} = ${platform}`;
        const [countResult] = await db.select({ count: sql `count(*)` }).from(downloadVersions).where(where);
        const total = Number(countResult?.count ?? 0);
        const rows = await db
            .select()
            .from(downloadVersions)
            .where(where)
            .orderBy(desc(downloadVersions.createdAt))
            .limit(size)
            .offset(offset);
        return paginated(rows, pageNum, size, total);
    });
    // POST /api/admin/downloads
    app.post("/downloads", { preHandler: [requireAdmin] }, async (request, reply) => {
        const { version, platform, fileKey, changelog, accessLevel, isLatest, isActive } = request.body;
        const [created] = await db
            .insert(downloadVersions)
            .values({
            version,
            platform,
            fileKey,
            changelog,
            accessLevel: accessLevel || "public",
            isLatest: isLatest || false,
            isActive: isActive !== false,
        })
            .returning();
        return ok(created);
    });
    // GET /api/admin/downloads/:id
    app.get("/downloads/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
        const { id } = request.params;
        const [dl] = await db.select().from(downloadVersions).where(eq(downloadVersions.id, id)).limit(1);
        if (!dl)
            return error("NOT_FOUND", "Download version not found", 404);
        return ok(dl);
    });
    // PATCH /api/admin/downloads/:id
    app.patch("/downloads/:id", { preHandler: [requireAdmin] }, async (request, reply) => {
        const { id } = request.params;
        const { version, platform, fileKey, changelog, accessLevel, isLatest, isActive } = request.body;
        const [updated] = await db
            .update(downloadVersions)
            .set({ version, platform, fileKey, changelog, accessLevel, isLatest, isActive, updatedAt: new Date() })
            .where(eq(downloadVersions.id, id))
            .returning();
        if (!updated)
            return error("NOT_FOUND", "Download version not found", 404);
        return ok(updated);
    });
}
//# sourceMappingURL=downloads.js.map