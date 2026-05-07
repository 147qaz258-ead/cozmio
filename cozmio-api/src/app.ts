import Fastify from "fastify";
import cors from "@fastify/cors";
import cookie from "@fastify/cookie";

export async function buildApp() {
  const app = Fastify({ logger: true });

  await app.register(cors, {
    origin: process.env.CORS_ORIGIN?.split(",") || ["http://localhost:3000"],
    credentials: true,
  });

  await app.register(cookie, {
    parseOptions: {},
  });

  return app;
}