"use client";

import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { projects } from "@/lib/network-data";
import { useLocale } from "@/lib/i18n";

export default function ProjectsPage() {
  const { locale, t } = useLocale();

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />
      <main className="flex-1 py-14 sm:py-18">
        <Container>
          <span className="section-kicker">{t.page.projectsKicker}</span>
          <h1 className="mt-6 max-w-[54rem] text-4xl font-semibold leading-tight text-primary-text sm:text-5xl">
            {t.page.projectsTitle}
          </h1>
          <p className="mt-5 max-w-[46rem] text-base leading-8 text-secondary-text">
            {t.page.projectsBody}
          </p>

          <div className="mt-10 grid gap-5 lg:grid-cols-3">
            {projects.map((project) => (
              <article key={project.name.en} className="rounded-xl border border-black/7 bg-white/76 p-6 shadow-[0_18px_55px_rgba(45,42,38,0.06)]">
                <div className="flex items-start justify-between gap-3">
                  <h2 className="text-xl font-semibold leading-7 text-primary-text">{project.name[locale]}</h2>
                  <span className="rounded-full border border-sage-green/20 bg-sage-green/12 px-2.5 py-1 text-xs font-semibold text-primary-text">
                    {project.status[locale]}
                  </span>
                </div>
                <p className="mt-4 text-sm leading-7 text-secondary-text">{project.outcome[locale]}</p>

                <dl className="mt-6 space-y-4 text-sm">
                  <div>
                    <dt className="font-semibold text-primary-text">{t.page.needed}</dt>
                    <dd className="mt-1 text-secondary-text">{project.needed[locale]}</dd>
                  </div>
                  <div>
                    <dt className="font-semibold text-primary-text">{t.page.budget}</dt>
                    <dd className="mt-1 text-secondary-text">{project.budget[locale]}</dd>
                  </div>
                  <div>
                    <dt className="font-semibold text-primary-text">{t.page.publicCase}</dt>
                    <dd className="mt-1 text-secondary-text">{project.publicCase[locale]}</dd>
                  </div>
                </dl>
              </article>
            ))}
          </div>
        </Container>
      </main>
      <Footer />
    </div>
  );
}
