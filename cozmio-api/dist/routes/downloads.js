import { eq, and } from "drizzle-orm";
import { db } from "../db/index.js";
import { downloadVersions } from "../db/schema.js";
import { ok, error } from "../lib/response.js";
import { S3Client, GetObjectCommand } from "@aws-sdk/client-s3";
import { getSignedUrl } from "@aws-sdk/s3-request-presigner";
const s3 = new S3Client({
    region: "auto",
    endpoint: `https://${process.env.R2_ACCOUNT_ID}.r2.cloudflarestorage.com`,
    credentials: {
        accessKeyId: process.env.R2_ACCESS_KEY_ID,
        secretAccessKey: process.env.R2_SECRET_ACCESS_KEY,
    },
});
export async function publicDownloadsRoutes(app) {
    // GET /api/downloads/latest?platform=windows
    app.get("/downloads/latest", async (request, reply) => {
        const { platform = "windows" } = request.query;
        const [version] = await db
            .select()
            .from(downloadVersions)
            .where(and(eq(downloadVersions.platform, platform), eq(downloadVersions.isActive, true), eq(downloadVersions.isLatest, true)))
            .limit(1);
        if (!version) {
            return error("NOT_FOUND", "No download available for this platform", 404);
        }
        // Generate R2 signed URL (15 min expiry)
        const signedUrl = await getSignedUrl(s3, new GetObjectCommand({
            Bucket: process.env.R2_BUCKET,
            Key: version.fileKey,
        }), { expiresIn: 900 });
        return ok({
            version: version.version,
            platform: version.platform,
            changelog: version.changelog,
            downloadUrl: signedUrl,
            accessLevel: version.accessLevel,
        });
    });
}
//# sourceMappingURL=downloads.js.map