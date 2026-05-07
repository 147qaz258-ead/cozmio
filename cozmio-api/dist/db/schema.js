import { pgTable, uuid, text, timestamp, boolean, jsonb, pgEnum } from "drizzle-orm/pg-core";
// Enums
export const userRoleEnum = pgEnum("user_role", ["user", "admin"]);
export const tokenTypeEnum = pgEnum("token_type", ["otp", "magic_link", "session"]);
export const applicationStatusEnum = pgEnum("application_status", ["new", "reviewed", "invited", "rejected"]);
export const taskStatusEnum = pgEnum("task_status", ["submitted", "queued", "processing", "needs_review", "done", "failed", "cancelled"]);
export const taskEventTypeEnum = pgEnum("task_event_type", ["created", "status_change", "note_added", "email_sent", "result_added"]);
export const platformEnum = pgEnum("platform", ["windows", "macos", "linux"]);
export const accessLevelEnum = pgEnum("access_level", ["public", "beta", "desktop", "hardware"]);
// Tables
export const users = pgTable("users", {
    id: uuid("id").defaultRandom().primaryKey(),
    email: text("email").unique().notNull(),
    name: text("name"),
    role: userRoleEnum("role").default("user").notNull(),
    webAccess: boolean("web_access").default(true).notNull(),
    betaAccess: boolean("beta_access").default(false).notNull(),
    desktopAccess: boolean("desktop_access").default(false).notNull(),
    hardwareAccess: boolean("hardware_access").default(false).notNull(),
    inviteCode: text("invite_code").unique(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
export const sessions = pgTable("sessions", {
    id: uuid("id").defaultRandom().primaryKey(),
    userId: uuid("user_id").references(() => users.id, { onDelete: "cascade" }),
    email: text("email").notNull(),
    token: text("token").unique().notNull(),
    tokenType: tokenTypeEnum("token_type").notNull(),
    expiresAt: timestamp("expires_at").notNull(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
});
export const applications = pgTable("applications", {
    id: uuid("id").defaultRandom().primaryKey(),
    name: text("name").notNull(),
    email: text("email").notNull(),
    company: text("company"),
    role: text("role"),
    useCase: text("use_case"),
    source: text("source"),
    status: applicationStatusEnum("status").default("new").notNull(),
    adminNote: text("admin_note"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
export const waitlist = pgTable("waitlist", {
    id: uuid("id").defaultRandom().primaryKey(),
    email: text("email").unique().notNull(),
    name: text("name"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
});
export const tasks = pgTable("tasks", {
    id: uuid("id").defaultRandom().primaryKey(),
    userId: uuid("user_id").references(() => users.id, { onDelete: "set null" }),
    email: text("email").notNull(),
    title: text("title").notNull(),
    prompt: text("prompt").notNull(),
    sourceUrl: text("source_url"),
    sourceType: text("source_type").default("manual"),
    status: taskStatusEnum("status").default("submitted").notNull(),
    resultSummary: text("result_summary"),
    resultPayload: jsonb("result_payload"),
    errorMessage: text("error_message"),
    internalNote: text("internal_note"),
    shareToken: text("share_token").unique(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
export const taskEvents = pgTable("task_events", {
    id: uuid("id").defaultRandom().primaryKey(),
    taskId: uuid("task_id").references(() => tasks.id, { onDelete: "cascade" }).notNull(),
    eventType: taskEventTypeEnum("event_type").notNull(),
    message: text("message"),
    metadata: jsonb("metadata"),
    createdAt: timestamp("created_at").defaultNow().notNull(),
});
export const downloadVersions = pgTable("download_versions", {
    id: uuid("id").defaultRandom().primaryKey(),
    version: text("version").notNull(),
    platform: platformEnum("platform").notNull(),
    fileKey: text("file_key").notNull(),
    changelog: text("changelog"),
    accessLevel: accessLevelEnum("access_level").default("public").notNull(),
    isLatest: boolean("is_latest").default(false).notNull(),
    isActive: boolean("is_active").default(true).notNull(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
    updatedAt: timestamp("updated_at").defaultNow().notNull(),
});
export const adminSessions = pgTable("admin_sessions", {
    id: uuid("id").defaultRandom().primaryKey(),
    email: text("email").notNull(),
    token: text("token").unique().notNull(),
    expiresAt: timestamp("expires_at").notNull(),
    createdAt: timestamp("created_at").defaultNow().notNull(),
});
//# sourceMappingURL=schema.js.map