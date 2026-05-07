import type { ElementType, ReactNode } from "react";
import Link from "next/link";
import type { LucideIcon } from "lucide-react";

type Tone = "violet" | "green" | "orange" | "gold" | "blue" | "dark";

const toneClasses: Record<Tone, string> = {
  violet: "from-[#eee8ff] text-[#6d54e8] shadow-[0_18px_36px_rgba(138,108,255,.22)]",
  green: "from-[#e9fae9] text-[#20a944] shadow-[0_18px_36px_rgba(49,196,86,.2)]",
  orange: "from-[#fff0e2] text-[#f17623] shadow-[0_18px_36px_rgba(255,139,61,.2)]",
  gold: "from-[#fff5dc] text-[#d9961d] shadow-[0_18px_36px_rgba(245,181,68,.22)]",
  blue: "from-[#e9f2ff] text-[#347be8] shadow-[0_18px_36px_rgba(106,167,255,.18)]",
  dark: "from-[#232323] text-white shadow-[0_18px_36px_rgba(0,0,0,.18)]",
};

export function CozButton({
  children,
  href,
  variant = "dark",
  className = "",
  type = "button",
}: {
  children: ReactNode;
  href?: string;
  variant?: "dark" | "light";
  className?: string;
  type?: "button" | "submit";
}) {
  const classes = `coz-button ${variant === "dark" ? "coz-btn-dark" : "coz-btn-light"} ${className}`;
  if (href) {
    if (href.startsWith("http")) {
      return (
        <a href={href} className={classes}>
          {children}
        </a>
      );
    }
    return (
      <Link href={href} className={classes}>
        {children}
      </Link>
    );
  }
  return <button type={type} className={classes}>{children}</button>;
}

export function CozBadge({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <span className={`coz-badge inline-flex items-center gap-2 px-4 py-2 text-xs font-semibold text-[#322d27] ${className}`}>{children}</span>;
}

export function CozCard({
  children,
  className = "",
  as: Component = "div",
}: {
  children: ReactNode;
  className?: string;
  as?: ElementType;
}) {
  return <Component className={`coz-card ${className}`}>{children}</Component>;
}

export function CozSection({ children, className = "" }: { children: ReactNode; className?: string }) {
  return <section className={`coz-shell relative ${className}`}>{children}</section>;
}

export function CozIconOrb({
  icon: Icon,
  tone = "gold",
  className = "",
}: {
  icon: LucideIcon;
  tone?: Tone;
  className?: string;
}) {
  return (
    <span className={`inline-flex h-16 w-16 items-center justify-center rounded-3xl bg-gradient-to-br to-white ${toneClasses[tone]} ${className}`}>
      <Icon className="h-7 w-7" />
    </span>
  );
}

export function CozStatus({ children, color = "green" }: { children: ReactNode; color?: "green" | "orange" | "gray" }) {
  const dot = color === "green" ? "bg-[#31c456]" : color === "orange" ? "bg-[#ff9d45]" : "bg-[#9b958e]";
  return (
    <span className="inline-flex items-center gap-2 text-xs font-medium text-[#67615a]">
      <span className={`h-2 w-2 rounded-full ${dot}`} />
      {children}
    </span>
  );
}

export function CozStat({ value, label, hint }: { value: string; label: string; hint: string }) {
  return (
    <div className="flex items-center gap-5">
      <div className="h-16 w-16 rounded-3xl bg-white/75 shadow-[0_16px_34px_rgba(44,34,22,.08)]" />
      <div>
        <div className="text-3xl font-bold tracking-tight text-[#151515]">{value}</div>
        <div className="mt-1 text-sm text-[#5f5952]">{label}</div>
        <div className="mt-1 text-sm font-semibold text-[#31a64a]">{hint}</div>
      </div>
    </div>
  );
}
