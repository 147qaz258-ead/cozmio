import { FastifyInstance } from "fastify";
import { createAdminSession, destroyAdminSession, verifyAdminPassword } from "../../services/admin-session.js";
import { ok, error } from "../../lib/response.js";

export async function adminAuthRoutes(app: FastifyInstance) {
  // POST /api/admin/auth/login
  app.post("/login", async (request, reply) => {
    const { email, password } = request.body as { email: string; password: string };

    if (!email || !password) {
      return error("VALIDATION_ERROR", "Email and password are required", 400);
    }

    const isValid = await verifyAdminPassword(email, password);
    if (!isValid) {
      return error("UNAUTHORIZED", "Invalid credentials", 401);
    }

    const { token, expiresAt } = await createAdminSession(email);

    reply.setCookie("admin_token", token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      expires: expiresAt,
    });

    return ok({ message: "Login successful" });
  });

  // POST /api/admin/auth/logout
  app.post("/logout", async (request, reply) => {
    const token = request.cookies.admin_token;
    if (token) {
      await destroyAdminSession(token);
    }
    reply.clearCookie("admin_token", { path: "/" });
    return ok({ message: "Logged out" });
  });

  // GET /api/admin/auth/me
  app.get("/me", async (request, reply) => {
    const token = request.cookies.admin_token || request.headers.authorization?.replace("Bearer ", "");
    if (!token) {
      return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Not authenticated" } });
    }

    const { validateAdminSession } = await import("../../services/admin-session.js");
    const admin = await validateAdminSession(token);
    if (!admin) {
      return reply.status(401).send({ ok: false, error: { code: "UNAUTHORIZED", message: "Session expired" } });
    }

    return ok({ email: admin.email });
  });
}