import { validateSession } from "../services/session.js";
export async function requireAuth(request, reply) {
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
//# sourceMappingURL=require-auth.js.map