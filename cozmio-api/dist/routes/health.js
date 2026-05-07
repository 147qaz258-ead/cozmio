export async function healthRoutes(app) {
    app.get("/health", async () => ({ ok: true }));
}
//# sourceMappingURL=health.js.map