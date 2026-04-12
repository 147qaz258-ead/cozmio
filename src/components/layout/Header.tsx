"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Container } from "@/components/layout/Container";
import { siteConfig } from "@/lib/site-config";
import { cn } from "@/lib/utils";

export function Header() {
  const pathname = usePathname();

  return (
    <header className="sticky top-0 z-50 w-full border-b border-black/6 bg-warm-white/82 backdrop-blur-xl">
      <Container className="py-3">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <Link href="/" className="flex items-center gap-3">
            <span className="inline-flex h-10 w-10 items-center justify-center rounded-2xl border border-black/8 bg-white/75 text-sm font-semibold text-primary-text shadow-[0_12px_28px_rgba(45,42,38,0.08)]">
              P
            </span>
            <span className="space-y-0.5">
              <span className="block text-base font-semibold text-primary-text">Pulseclaw</span>
              <span className="block text-[11px] uppercase tracking-[0.18em] text-secondary-text/75">
                {siteConfig.tagline}
              </span>
            </span>
          </Link>

          <div className="hidden rounded-full border border-mist-blue/16 bg-white/72 px-3 py-1.5 text-[11px] font-medium text-secondary-text shadow-[0_8px_24px_rgba(45,42,38,0.05)] xl:flex xl:items-center xl:gap-2">
            <span className="inline-flex h-2 w-2 rounded-full bg-sage-green ambient-pulse" />
            Desktop-first public site
          </div>

          <div className="flex items-center gap-2.5">
            <a
              href={siteConfig.links.github}
              target="_blank"
              rel="noreferrer"
              className="hidden rounded-full border border-black/8 bg-white/74 px-4 py-2 text-sm font-medium text-primary-text transition-colors hover:bg-white md:inline-flex"
            >
              GitHub
            </a>
            <Link
              href="/demo"
              className="inline-flex rounded-full bg-primary-text px-4 py-2 text-sm font-semibold text-white shadow-[0_14px_28px_rgba(45,42,38,0.16)] transition-colors hover:bg-primary-text/94"
            >
              Run Demo
            </Link>
          </div>
        </div>

        <nav className="mt-3 flex gap-2 overflow-x-auto pb-1 md:hidden">
          {siteConfig.navItems.map((item) => {
            const active =
              item.href === "/"
                ? pathname === "/"
                : pathname === item.href || pathname.startsWith(`${item.href}/`);

            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  "shrink-0 rounded-full border px-3 py-2 text-sm font-medium transition-colors",
                  active
                    ? "border-mist-blue/24 bg-white text-primary-text"
                    : "border-black/6 bg-white/62 text-secondary-text hover:bg-white/82"
                )}
              >
                {item.label}
              </Link>
            );
          })}
        </nav>

        <nav className="mt-3 hidden items-center gap-3 text-sm font-medium md:flex">
          {siteConfig.navItems.map((item) => {
            const active =
              item.href === "/"
                ? pathname === "/"
                : pathname === item.href || pathname.startsWith(`${item.href}/`);

            return (
              <Link
                key={item.href}
                href={item.href}
                className={cn(
                  "rounded-full px-3 py-2 transition-colors",
                  active
                    ? "bg-white text-primary-text shadow-[0_10px_22px_rgba(45,42,38,0.05)]"
                    : "text-secondary-text hover:text-primary-text"
                )}
              >
                {item.label}
              </Link>
            );
          })}
        </nav>
      </Container>
    </header>
  );
}
