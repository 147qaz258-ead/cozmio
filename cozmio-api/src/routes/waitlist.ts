import { FastifyInstance } from "fastify";
import { db } from "../db/index.js";
import { waitlist } from "../db/schema.js";
import { ok, error } from "../lib/response.js";

export async function publicWaitlistRoutes(app: FastifyInstance) {
  // POST /api/waitlist
  app.post("/waitlist", async (request, reply) => {
    const { email, name } = request.body as any;

    if (!email || !email.includes("@")) {
      return error("VALIDATION_ERROR", "Valid email is required", 400);
    }

    try {
      const [created] = await db
        .insert(waitlist)
        .values({ email, name: name || null })
        .returning();
      return ok(created);
    } catch (err: any) {
      // Unique constraint violation means already subscribed
      if (err?.code === "23505") {
        return ok({ message: "Already subscribed" });
      }
      throw err;
    }
  });
}