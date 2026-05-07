import { sendOtp, verifyOtp, destroySession } from "../services/session.js";
import { sendOtpEmail } from "../services/email.js";
import { ok, error } from "../lib/response.js";
import { requireAuth } from "../middleware/require-auth.js";
export async function authRoutes(app) {
    app.post("/send-code", async (request, reply) => {
        const { email, type } = request.body;
        if (!email || !email.includes("@")) {
            return error("VALIDATION_ERROR", "Invalid email", 400);
        }
        const code = await sendOtp(email);
        if (process.env.NODE_ENV !== "test") {
            await sendOtpEmail(email, code);
        }
        return ok({ message: "Verification code sent" });
    });
    app.post("/verify", async (request, reply) => {
        const { email, token } = request.body;
        if (!email || !token) {
            return error("VALIDATION_ERROR", "Email and token are required", 400);
        }
        const { user, sessionToken, expiresAt } = await verifyOtp(email, token);
        reply.setCookie("session_token", sessionToken, {
            httpOnly: true,
            secure: process.env.NODE_ENV === "production",
            sameSite: "lax",
            path: "/",
            expires: expiresAt,
        });
        return ok({ user, session_token: sessionToken, expires_at: expiresAt });
    });
    app.post("/logout", async (request, reply) => {
        const token = request.cookies.session_token;
        if (token) {
            await destroySession(token);
        }
        reply.clearCookie("session_token", { path: "/" });
        return ok({ message: "Logged out" });
    });
    app.get("/me", { preHandler: [requireAuth] }, async (request, reply) => {
        return ok(request.user);
    });
}
//# sourceMappingURL=auth.js.map