import { FastifyRequest, FastifyReply } from "fastify";
import { validateAdminSession } from "../services/admin-session.js";
declare module "fastify" {
    interface FastifyRequest {
        admin?: Awaited<ReturnType<typeof validateAdminSession>>;
    }
}
export declare function requireAdmin(request: FastifyRequest, reply: FastifyReply): Promise<undefined>;
//# sourceMappingURL=require-admin.d.ts.map