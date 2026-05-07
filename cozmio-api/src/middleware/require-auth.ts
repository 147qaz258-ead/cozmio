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