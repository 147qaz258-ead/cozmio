import { buildApp } from "./app.js";
import { AppError } from "./lib/errors.js";
import { registerRoutes } from "./routes/index.js";
import { registerAdminRoutes } from "./routes/admin/index.js";
const app = await buildApp();
app.get("/health", async () => ({ ok: true }));
// Register routes
await registerRoutes(app);
await registerAdminRoutes(app);
// Global error handler
app.setErrorHandler((error, request, reply) => {
    if (error instanceof AppError) {
        return reply.status(error.status).send({ ok: false, error: { code: error.code, message: error.message } });
    }
    request.log.error(error);
    return reply.status(500).send({ ok: false, error: { code: "INTERNAL_ERROR", message: "Internal server error" } });
});
const port = Number(process.env.PORT) || 3000;
await app.listen({ port, host: "0.0.0.0" });
console.log(`Server listening on port ${port}`);
//# sourceMappingURL=index.js.map