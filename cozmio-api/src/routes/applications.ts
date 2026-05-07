import { FastifyInstance } from "fastify";
import { db } from "../db/index.js";
import { applications } from "../db/schema.js";
import { ok, error } from "../lib/response.js";

export async function publicApplicationsRoutes(app: FastifyInstance) {
  // POST /api/applications - public application
  app.post("/applications", async (request, reply) => {
    const { name, email, company, role, useCase, source } = request.body as any;

    if (!name || !email || !useCase) {
      return error("VALIDATION_ERROR", "Name, email and useCase are required", 400);
    }

    if (!email.includes("@")) {
      return error("VALIDATION_ERROR", "Invalid email format", 400);
    }

    const [created] = await db
      .insert(applications)
      .values({
        name,
        email,
        company: company || null,
        role: role || null,
        useCase,
        source: source || "website",
        status: "new",
      })
      .returning();

    return ok(created);
  });
}