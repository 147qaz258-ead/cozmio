"use client";

import Link from "next/link";
import { Container } from "@/components/layout/Container";
import { footerGroups, siteConfig } from "@/lib/site-config";
import { useLocale } from "@/lib/i18n";
import { useRef, FormEvent } from "react";

export function Footer() {
  const { t } = useLocale();
  const subscribeEmailRef = useRef<HTMLInputElement>(null);

  const handleSubscribe = async (e: FormEvent) => {
    e.preventDefault();
    const email = subscribeEmailRef.current?.value;
    if (!email) return;
    try {
      const res = await fetch('/api/waitlist', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ email, source: 'footer-subscribe' }),
      });
      if (res.ok) {
        alert('订阅成功！');
        if (subscribeEmailRef.current) subscribeEmailRef.current.value = '';
      } else {
        alert('订阅失败，请稍后再试');
      }
    } catch {
      alert('订阅失败，请稍后再试');
    }
  };

  return (
    <footer className="border-t border-black/6 bg-white/55 py-10 backdrop-blur-sm sm:py-12">
      <Container>
        <div className="grid gap-10 border-b border-black/6 pb-8 lg:grid-cols-[1.15fr_0.85fr_0.85fr_0.85fr_0.85fr]">
          <div className="max-w-[26rem]">
            <div className="text-base font-semibold text-primary-text">{siteConfig.name}</div>
            <p className="mt-3 text-sm leading-7 text-secondary-text">{t.footer.description}</p>
            <p className="mt-4 text-sm leading-7 text-secondary-text">{t.footer.desktopNote}</p>
            <form onSubmit={handleSubscribe} className="mt-4 flex gap-2">
              <input
                ref={subscribeEmailRef}
                type="email"
                placeholder="输入邮箱订阅更新"
                className="flex-1 rounded border border-black/10 px-3 py-1.5 text-sm"
                required
              />
              <button type="submit" className="rounded bg-primary px-3 py-1.5 text-sm text-white">
                订阅
              </button>
            </form>
          </div>

          {footerGroups.map((group) => (
            <div key={group.title}>
              <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/68">
                {t.footer.groups[group.title]}
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
                      {t.footer.links[link.label]}
                    </a>
                  ) : (
                    <Link
                      key={link.label}
                      href={link.href}
                      className="block text-sm text-secondary-text transition-colors hover:text-primary-text"
                    >
                      {t.footer.links[link.label]}
                    </Link>
                  )
                )}
              </div>
            </div>
          ))}
        </div>

        <div className="flex flex-col gap-3 pt-5 text-sm text-secondary-text sm:flex-row sm:items-center sm:justify-between">
          <p>© 2026 Cozmio. All rights reserved.</p>
          <div className="flex flex-wrap gap-4">
            <a href={siteConfig.links.email} className="transition-colors hover:text-primary-text">
              {siteConfig.email}
            </a>
            <a href={siteConfig.links.github} target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
              {t.nav.github}
            </a>
            <a href="https://x.com/wjnhng419090" target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
              X
            </a>
            <a href="https://discord.gg/cozmio" target="_blank" rel="noreferrer" className="transition-colors hover:text-primary-text">
              DC
            </a>
          </div>
        </div>
      </Container>
    </footer>
  );
}
