import { FastifyInstance } from "fastify";
import { authRoutes } from "./auth.js";
import { tasksRoutes } from "./tasks.js";
import { publicApplicationsRoutes } from "./applications.js";
import { publicDownloadsRoutes } from "./downloads.js";
import { publicWaitlistRoutes } from "./waitlist.js";

export async function registerRoutes(app: FastifyInstance) {
  app.register(authRoutes, { prefix: "/api/auth" });
  app.register(tasksRoutes, { prefix: "/api/tasks" });
  // Public routes
  app.register(publicApplicationsRoutes, { prefix: "/api" });
  app.register(publicDownloadsRoutes, { prefix: "/api" });
  app.register(publicWaitlistRoutes, { prefix: "/api" });
}