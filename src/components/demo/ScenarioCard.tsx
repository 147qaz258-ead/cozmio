"use client";

import Link from "next/link";
import { cn } from "@/lib/utils";

export interface ScenarioCardProps {
  icon: string;
  title: string;
  description: string;
  status: "featured" | "active" | "coming-soon";
  href?: string;
  badge?: string;
  detail?: string;
  ctaLabel?: string;
}

function ScenarioCardBody({
  icon,
  title,
  description,
  status,
  badge,
  detail,
  ctaLabel,
}: Omit<ScenarioCardProps, "href">) {
  const interactive = status === "featured" || status === "active";

  return (
    <div
      className={cn(
        "surface-panel relative flex h-full flex-col rounded-[1.75rem] p-5 transition-all duration-300",
        interactive
          ? "hover:-translate-y-1 hover:shadow-[0_28px_72px_rgba(45,42,38,0.12)]"
          : "opacity-78",
        status === "featured" && "surface-panel-strong",
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="inline-flex h-12 w-12 items-center justify-center rounded-2xl border border-black/6 bg-white/86 text-2xl shadow-[0_14px_28px_rgba(45,42,38,0.05)]">
          {icon}
        </div>
        <span
          className={cn(
            "rounded-full border px-3 py-1 text-[11px] font-semibold uppercase tracking-[0.14em]",
            status === "featured"
              ? "border-mist-blue/18 bg-mist-blue/10 text-mist-blue"
              : status === "active"
                ? "border-sage-green/18 bg-sage-green/10 text-sage-green"
                : "border-black/6 bg-white/72 text-secondary-text/70",
          )}
        >
          {badge ?? (status === "coming-soon" ? "soon" : "available")}
        </span>
      </div>

      <div className="mt-5">
        <h3 className="text-[1.4rem] font-semibold leading-8 text-primary-text">
          {title}
        </h3>
        <p className="mt-3 text-sm leading-7 text-secondary-text">
          {description}
        </p>
      </div>

      <div className="mt-6 rounded-[1.3rem] border border-black/6 bg-white/78 px-4 py-3 text-sm leading-6 text-primary-text">
        {detail}
      </div>

      <div className="mt-6 flex items-center justify-between">
        <span className="text-[11px] font-semibold uppercase tracking-[0.14em] text-secondary-text/66">
          {interactive ? "available now" : "coming next"}
        </span>
        <span
          className={cn(
            "inline-flex items-center gap-2 rounded-full px-3 py-1.5 text-sm font-medium",
            interactive
              ? "bg-primary-text text-white"
              : "bg-warm-card text-secondary-text",
          )}
        >
          {interactive ? ctaLabel ?? "打开" : "即将推出"}
          {interactive && <span aria-hidden>→</span>}
        </span>
      </div>
    </div>
  );
}

export function ScenarioCard({ href, ...props }: ScenarioCardProps) {
  const interactive = (props.status === "featured" || props.status === "active") && href;

  if (interactive) {
    return (
      <Link href={href} className="block h-full">
        <ScenarioCardBody {...props} />
      </Link>
    );
  }

  return <ScenarioCardBody {...props} />;
}
