import { FastifyInstance } from "fastify";
import { adminAuthRoutes } from "./auth.js";
import { adminApplicationsRoutes } from "./applications.js";
import { adminTasksRoutes } from "./tasks.js";
import { adminUsersRoutes } from "./users.js";
import { adminDownloadsRoutes } from "./downloads.js";

export async function registerAdminRoutes(app: FastifyInstance) {
  app.register(adminAuthRoutes, { prefix: "/auth" });
  app.register(adminApplicationsRoutes);
  app.register(adminTasksRoutes);
  app.register(adminUsersRoutes);
  app.register(adminDownloadsRoutes);
}