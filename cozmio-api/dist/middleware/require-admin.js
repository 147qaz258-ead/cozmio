import { validateAdminSession } from "../services/admin-session.js";
export async function requireAdmin(request, reply) {
    const token = request.cookies.admin_token || request.headers.authorization?.replace("Bearer ", "");
    if (!token) {
        return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Admin not authenticated" } });
    }
    const admin = await validateAdminSession(token);
    if (!admin) {
        return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Admin session expired" } });
    }
    request.admin = admin;
}
//# sourceMappingURL=require-admin.js.map