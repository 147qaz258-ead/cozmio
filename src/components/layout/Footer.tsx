import Link from "next/link";
import { Container } from "@/components/layout/Container";
import { footerGroups, siteConfig } from "@/lib/site-config";

export function Footer() {
  return (
    <footer className="border-t border-black/6 bg-white/55 py-10 backdrop-blur-sm sm:py-12">
      <Container>
        <div className="grid gap-10 border-b border-black/6 pb-8 lg:grid-cols-[1.15fr_0.85fr_0.85fr_0.85fr_0.85fr]">
          <div className="max-w-[26rem]">
            <div className="text-base font-semibold text-primary-text">{siteConfig.name}</div>
            <p className="mt-3 text-sm leading-7 text-secondary-text">{siteConfig.description}</p>
            <p className="mt-4 text-sm leading-7 text-secondary-text">{siteConfig.desktopNote}</p>
          </div>

          {footerGroups.map((group) => (
            <div key={group.title}>
              <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                {group.title}
              </div>
              <div className="mt-4 space-y-3">
                {group.links.map((link) =>
                  "external" in link && link.external ? (
                    <a
                      key={link.label}
                      href={link.href}
                      target={link.href.startsWith("mailto:") ? undefined : "_blank"}
                      rel={link.href.startsWith("mailto:") ? undefined : "noreferrer"}
                      className="block text-sm text-secondary-text transition-colors hover:text-primary-text"
                    >
                      {link.label}
                    </a>
                  ) : (
                    <Link
                      key={link.label}
                      href={link.href}
                      className="block text-sm text-secondary-text transition-colors hover:text-primary-text"
                    >
                      {link.label}
                    </Link>
                  )
                )}
              </div>
            </div>
          ))}
        </div>

        <div className="flex flex-col gap-3 pt-5 text-sm text-secondary-text sm:flex-row sm:items-center sm:justify-between">
          <p>© 2026 Pulseclaw. All rights reserved.</p>
          <div className="flex flex-wrap gap-4">
            <a href={siteConfig.links.email} className="transition-colors hover:text-primary-text">
              {siteConfig.email}
            </a>
            <a href={siteConfig.links.github} target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
              GitHub
            </a>
          </div>
        </div>
      </Container>
    </footer>
  );
}
