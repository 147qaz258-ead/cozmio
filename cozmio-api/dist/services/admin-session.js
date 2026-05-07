import { v4 as uuidv4 } from "uuid";
import { eq } from "drizzle-orm";
import { db } from "../db/index.js";
import { adminSessions } from "../db/schema.js";
import bcrypt from "bcrypt";
export async function createAdminSession(email) {
    const token = uuidv4();
    const expiresAt = new Date(Date.now() + 24 * 60 * 60 * 1000); // 24h
    await db.insert(adminSessions).values({
        email,
        token,
        expiresAt,
    });
    return { token, expiresAt };
}
export async function validateAdminSession(token) {
    const [session] = await db
        .select()
        .from(adminSessions)
        .where(eq(adminSessions.token, token))
        .limit(1);
    if (!session || session.expiresAt < new Date()) {
        // 清理过期 session
        if (session) {
            await db.delete(adminSessions).where(eq(adminSessions.id, session.id));
        }
        return null;
    }
    return { email: session.email };
}
export async function destroyAdminSession(token) {
    await db.delete(adminSessions).where(eq(adminSessions.token, token));
}
export async function verifyAdminPassword(email, password) {
    const adminEmail = process.env.ADMIN_EMAIL;
    const adminHash = process.env.ADMIN_PASSWORD_HASH;
    if (!adminEmail || !adminHash) {
        return false;
    }
    if (email !== adminEmail) {
        return false;
    }
    return bcrypt.compare(password, adminHash);
}
//# sourceMappingURL=admin-session.js.map