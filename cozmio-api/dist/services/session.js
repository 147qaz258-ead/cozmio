import { v4 as uuidv4 } from "uuid";
import { eq } from "drizzle-orm";
import { db } from "../db/index.js";
import { sessions, users } from "../db/schema.js";
import { ERRORS } from "../lib/errors.js";
export async function sendOtp(email) {
    // Generate 6-digit OTP
    const code = Math.floor(100000 + Math.random() * 900000).toString();
    const expiresAt = new Date(Date.now() + 10 * 60 * 1000); // 10 min
    // Find or create user
    let [user] = await db.select().from(users).where(eq(users.email, email)).limit(1);
    if (!user) {
        [user] = await db.insert(users).values({ email }).returning();
    }
    // Delete old OTP sessions for this email
    await db.delete(sessions).where(eq(sessions.email, email));
    // Create new OTP session
    await db.insert(sessions).values({
        userId: user.id,
        email,
        token: code,
        tokenType: "otp",
        expiresAt,
    });
    return code;
}
export async function verifyOtp(email, code) {
    const [session] = await db
        .select()
        .from(sessions)
        .where(eq(sessions.email, email))
        .limit(1);
    if (!session || session.token !== code || session.tokenType !== "otp" || session.expiresAt < new Date()) {
        throw ERRORS.UNAUTHORIZED("Invalid or expired verification code");
    }
    // Delete one-time OTP session
    await db.delete(sessions).where(eq(sessions.id, session.id));
    // Create persistent session (7 days)
    const sessionToken = uuidv4();
    const expiresAt = new Date(Date.now() + 7 * 24 * 60 * 60 * 1000);
    await db.insert(sessions).values({
        userId: session.userId,
        email,
        token: sessionToken,
        tokenType: "session",
        expiresAt,
    });
    const [user] = await db.select().from(users).where(eq(users.id, session.userId)).limit(1);
    return { user, sessionToken, expiresAt };
}
export async function validateSession(token) {
    const [session] = await db
        .select()
        .from(sessions)
        .where(eq(sessions.token, token))
        .limit(1);
    if (!session || session.tokenType !== "session" || session.expiresAt < new Date()) {
        return null;
    }
    const [user] = await db.select().from(users).where(eq(users.id, session.userId)).limit(1);
    return user || null;
}
export async function destroySession(token) {
    await db.delete(sessions).where(eq(sessions.token, token));
}
//# sourceMappingURL=session.js.map