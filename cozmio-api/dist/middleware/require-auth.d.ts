import { FastifyRequest, FastifyReply } from "fastify";
import { validateSession } from "../services/session.js";
declare module "fastify" {
    interface FastifyRequest {
        user?: Awaited<ReturnType<typeof validateSession>>;
    }
}
export declare function requireAuth(request: FastifyRequest, reply: FastifyReply): Promise<undefined>;
//# sourceMappingURL=require-auth.d.ts.map