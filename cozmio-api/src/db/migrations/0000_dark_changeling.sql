CREATE TYPE "public"."access_level" AS ENUM('public', 'beta', 'desktop', 'hardware');--> statement-breakpoint
CREATE TYPE "public"."application_status" AS ENUM('new', 'reviewed', 'invited', 'rejected');--> statement-breakpoint
CREATE TYPE "public"."platform" AS ENUM('windows', 'macos', 'linux');--> statement-breakpoint
CREATE TYPE "public"."task_event_type" AS ENUM('created', 'status_change', 'note_added', 'email_sent', 'result_added');--> statement-breakpoint
CREATE TYPE "public"."task_status" AS ENUM('submitted', 'queued', 'processing', 'needs_review', 'done', 'failed', 'cancelled');--> statement-breakpoint
CREATE TYPE "public"."token_type" AS ENUM('otp', 'magic_link');--> statement-breakpoint
CREATE TYPE "public"."user_role" AS ENUM('user', 'admin');--> statement-breakpoint
CREATE TABLE "admin_sessions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"email" text NOT NULL,
	"token" text NOT NULL,
	"expires_at" timestamp NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "admin_sessions_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "applications" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"name" text NOT NULL,
	"email" text NOT NULL,
	"company" text,
	"role" text,
	"use_case" text,
	"source" text,
	"status" "application_status" DEFAULT 'new' NOT NULL,
	"admin_note" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "download_versions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"version" text NOT NULL,
	"platform" "platform" NOT NULL,
	"file_key" text NOT NULL,
	"changelog" text,
	"access_level" "access_level" DEFAULT 'public' NOT NULL,
	"is_latest" boolean DEFAULT false NOT NULL,
	"is_active" boolean DEFAULT true NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "sessions" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"user_id" uuid,
	"email" text NOT NULL,
	"token" text NOT NULL,
	"token_type" "token_type" NOT NULL,
	"expires_at" timestamp NOT NULL,
	"created_at" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "sessions_token_unique" UNIQUE("token")
);
--> statement-breakpoint
CREATE TABLE "task_events" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"task_id" uuid NOT NULL,
	"event_type" "task_event_type" NOT NULL,
	"message" text,
	"metadata" jsonb,
	"created_at" timestamp DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "tasks" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"user_id" uuid,
	"email" text NOT NULL,
	"title" text NOT NULL,
	"prompt" text NOT NULL,
	"source_url" text,
	"source_type" text DEFAULT 'manual',
	"status" "task_status" DEFAULT 'submitted' NOT NULL,
	"result_summary" text,
	"result_payload" jsonb,
	"error_message" text,
	"internal_note" text,
	"share_token" text DEFAULT nanoid(16),
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "tasks_share_token_unique" UNIQUE("share_token")
);
--> statement-breakpoint
CREATE TABLE "users" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"email" text NOT NULL,
	"name" text,
	"role" "user_role" DEFAULT 'user' NOT NULL,
	"web_access" boolean DEFAULT true NOT NULL,
	"beta_access" boolean DEFAULT false NOT NULL,
	"desktop_access" boolean DEFAULT false NOT NULL,
	"hardware_access" boolean DEFAULT false NOT NULL,
	"invite_code" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	"updated_at" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "users_email_unique" UNIQUE("email"),
	CONSTRAINT "users_invite_code_unique" UNIQUE("invite_code")
);
--> statement-breakpoint
CREATE TABLE "waitlist" (
	"id" uuid PRIMARY KEY DEFAULT gen_random_uuid() NOT NULL,
	"email" text NOT NULL,
	"name" text,
	"created_at" timestamp DEFAULT now() NOT NULL,
	CONSTRAINT "waitlist_email_unique" UNIQUE("email")
);
--> statement-breakpoint
ALTER TABLE "sessions" ADD CONSTRAINT "sessions_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "task_events" ADD CONSTRAINT "task_events_task_id_tasks_id_fk" FOREIGN KEY ("task_id") REFERENCES "public"."tasks"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "tasks" ADD CONSTRAINT "tasks_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE set null ON UPDATE no action;