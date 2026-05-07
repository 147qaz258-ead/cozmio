import { authRoutes } from "./auth.js";
import { tasksRoutes } from "./tasks.js";
import { publicApplicationsRoutes } from "./applications.js";
import { publicDownloadsRoutes } from "./downloads.js";
import { publicWaitlistRoutes } from "./waitlist.js";
export async function registerRoutes(app) {
    app.register(authRoutes, { prefix: "/api/auth" });
    app.register(tasksRoutes, { prefix: "/api/tasks" });
    // Public routes
    app.register(publicApplicationsRoutes, { prefix: "/api" });
    app.register(publicDownloadsRoutes, { prefix: "/api" });
    app.register(publicWaitlistRoutes, { prefix: "/api" });
}
//# sourceMappingURL=index.js.map