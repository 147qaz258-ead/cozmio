"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Container } from "@/components/layout/Container";
import { siteConfig } from "@/lib/site-config";
import { useLocale } from "@/lib/i18n";
import { cn } from "@/lib/utils";

const navLabels = {
  "/agents": "agents",
  "/projects": "projects",
  "/cases": "cases",
  "/desktop": "desktop",
  "/request": "request",
} as const;

export function Header() {
  const pathname = usePathname();
  const { locale, setLocale, t } = useLocale();

  return (
    <header className="sticky top-0 z-50 w-full border-b border-black/6 bg-warm-white/82 backdrop-blur-xl">
      <Container className="py-3">
        <div className="flex flex-wrap items-center justify-between gap-4">
          <Link href="/" className="flex items-center gap-3">
            <span className="inline-flex h-10 w-10 items-center justify-center rounded-2xl border border-black/8 bg-white/75 text-sm font-semibold text-primary-text shadow-[0_12px_28px_rgba(45,42,38,0.08)]">
              C
            </span>
            <span className="space-y-0.5">
              <span className="block text-base font-semibold text-primary-text">{siteConfig.name}</span>
              <span className="block text-[11px] uppercase tracking-[0.18em] text-secondary-text/75">
                {siteConfig.tagline}
              </span>
            </span>
          </Link>

          <div className="hidden rounded-full border border-mist-blue/16 bg-white/72 px-3 py-1.5 text-[11px] font-medium text-secondary-text shadow-[0_8px_24px_rgba(45,42,38,0.05)] xl:flex xl:items-center xl:gap-2">
            <span className="inline-flex h-2 w-2 rounded-full bg-sage-green ambient-pulse" />
            {t.nav.beta}
          </div>

          <div className="flex items-center gap-2.5">
            <div className="inline-flex rounded-full border border-black/8 bg-white/72 p-1 text-xs font-semibold text-secondary-text">
              <button
                type="button"
                onClick={() => setLocale("zh")}
                className={cn("rounded-full px-2.5 py-1 transition-colors", locale === "zh" && "bg-primary-text text-white")}
              >
                中
              </button>
              <button
                type="button"
                onClick={() => setLocale("en")}
                className={cn("rounded-full px-2.5 py-1 transition-colors", locale === "en" && "bg-primary-text text-white")}
              >
                EN
              </button>
            </div>
            <a
              href={siteConfig.links.github}
              target="_blank"
              rel="noreferrer"
              className="hidden rounded-full border border-black/8 bg-white/74 px-4 py-2 text-sm font-medium text-primary-text transition-colors hover:bg-white md:inline-flex"
            >
              {t.nav.github}
            </a>
            <Link
              href="/request"
              className="inline-flex rounded-full bg-primary-text px-4 py-2 text-sm font-semibold text-white shadow-[0_14px_28px_rgba(45,42,38,0.16)] transition-colors hover:bg-primary-text/94"
            >
              {t.nav.submitTask}
            </Link>
          </div>
        </div>

        <nav className="mt-3 flex gap-2 overflow-x-auto pb-1 md:hidden">
          {siteConfig.navItems.map((item) => {
            const active = pathname === item.href || pathname.startsWith(`${item.href}/`);

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
                {t.nav[navLabels[item.href]]}
              </Link>
            );
          })}
        </nav>

        <nav className="mt-3 hidden items-center gap-3 text-sm font-medium md:flex">
          {siteConfig.navItems.map((item) => {
            const active = pathname === item.href || pathname.startsWith(`${item.href}/`);

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
                {t.nav[navLabels[item.href]]}
              </Link>
            );
          })}
        </nav>
      </Container>
    </header>
  );
}
