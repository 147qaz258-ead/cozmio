import type { Metadata } from "next";
import Link from "next/link";
import { BlogHeader } from "@/components/blog/BlogHeader";
import { BlogFooter } from "@/components/blog/BlogFooter";
import { getAllVolumes, getBookStats } from "@/lib/book";

export const metadata: Metadata = {
  title: "Writing — Pulseclaw 产品思考与更新",
  description:
    "Pulseclaw 的产品思考：上下文捕获、证据驱动的 AI 帮助，以及桌面端工具的设计记录。",
  alternates: { canonical: "/blog" },
};

export default function BlogIndexPage() {
  const volumes = getAllVolumes();
  const stats = getBookStats();

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <BlogHeader currentPath="/blog" />

      <main className="flex-1">
        {/* Hero Section */}
        <section className="relative overflow-hidden py-16 lg:py-24">
          {/* Background Pattern */}
          <div className="absolute inset-0 page-grid opacity-30" />

          <div className="relative mx-auto max-w-6xl px-6">
            <div className="max-w-3xl">
              {/* Badge */}
              <div className="mb-6 inline-flex items-center gap-2 rounded-full border border-mist-blue/20 bg-mist-blue/5 px-4 py-2">
                <span className="h-2 w-2 rounded-full bg-mist-blue animate-pulse" />
                <span className="text-sm font-medium text-mist-blue">开源书籍 · 持续更新</span>
              </div>

              {/* Title */}
              <h1 className="text-[clamp(2.4rem,5vw,3.6rem)] font-bold leading-tight text-primary-text">
                从零开始，
                <span className="text-mist-blue">掌握 AI Agent 框架。</span>
              </h1>

              {/* Description */}
              <p className="mt-6 text-lg leading-relaxed text-secondary-text max-w-2xl">
                这不是官方文档的翻译，也不是 API 参考手册。<br />
                这是一本写给实践者的书——每章都有可运行的代码和可验证的步骤。
              </p>

              {/* Stats */}
              <div className="mt-8 flex flex-wrap gap-6">
                <Stat value={stats.volumes.toString()} label="卷" />
                <Stat value={stats.totalChapters.toString()} label="章" />
                <Stat value={stats.totalWords} label="" />
              </div>

              {/* CTA */}
              <div className="mt-10">
                <Link
                  href="/book/vol1/ch-01"
                  className="inline-flex items-center gap-2 rounded-full bg-gradient-to-r from-mist-blue to-sage-green px-8 py-4 text-white font-semibold shadow-lg shadow-mist-blue/20 transition-transform hover:scale-105"
                >
                  开始阅读
                  <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 8l4 4m0 0l-4 4m4-4H3" />
                  </svg>
                </Link>
              </div>
            </div>
          </div>
        </section>

        {/* Volumes Section */}
        <section className="py-12 lg:py-16">
          <div className="mx-auto max-w-6xl px-6">
            {volumes.map((volume, index) => (
              <div key={volume.id} className="mb-12 last:mb-0">
                {/* Volume Header */}
                <div className="mb-6 flex items-end justify-between border-b border-black/6 pb-4">
                  <div>
                    <div className="text-xs font-semibold uppercase tracking-widest text-secondary-text/50 mb-1">
                      {volume.id}
                    </div>
                    <h2 className="text-2xl font-bold text-primary-text">
                      {volume.title}
                    </h2>
                  </div>
                  <div className="text-sm text-secondary-text">
                    {volume.chapters.length} 章
                  </div>
                </div>

                {/* Chapters Grid */}
                <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
                  {volume.chapters.map((chapter) => (
                    <Link
                      key={chapter.slug}
                      href={`/book/${volume.id}/${chapter.slug}`}
                      className="group relative overflow-hidden rounded-2xl border border-black/6 bg-white p-5 transition-all hover:border-mist-blue/30 hover:shadow-lg hover:shadow-mist-blue/5"
                    >
                      {/* Chapter Number */}
                      <div className="absolute -right-4 -top-4 text-[5rem] font-bold text-black/[0.03] transition-colors group-hover:text-mist-blue/[0.08]">
                        {chapter.chapterIndex || index + 1}
                      </div>

                      {/* Content */}
                      <div className="relative">
                        <div className="text-xs font-medium text-mist-blue mb-2">
                          第 {chapter.chapterIndex || '?'} 章
                        </div>
                        <h3 className="font-semibold text-primary-text line-clamp-2 group-hover:text-mist-blue transition-colors">
                          {chapter.title}
                        </h3>
                        {chapter.status === "draft" && (
                          <span className="mt-3 inline-block rounded-full bg-amber-50 px-2 py-0.5 text-[10px] font-medium text-amber-600">
                            Draft
                          </span>
                        )}
                      </div>

                      {/* Arrow */}
                      <div className="mt-4 flex items-center text-sm font-medium text-secondary-text/50 transition-colors group-hover:text-mist-blue">
                        阅读
                        <svg className="ml-1 h-4 w-4 transition-transform group-hover:translate-x-1" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                        </svg>
                      </div>
                    </Link>
                  ))}
                </div>
              </div>
            ))}
          </div>
        </section>

        {/* About Section */}
        <section className="py-16 bg-gradient-to-b from-transparent to-mist-blue/5">
          <div className="mx-auto max-w-6xl px-6">
            <div className="rounded-3xl border border-mist-blue/10 bg-white/60 p-8 lg:p-12">
              <h2 className="text-2xl font-bold text-primary-text mb-4">
                为什么写这本书？
              </h2>
              <div className="prose prose-lg max-w-none text-secondary-text">
                <p>
                  AI 正在从&quot;你问它答&quot;走向&quot;它替你做&quot;。OpenClaw 是这条路上的一个岔路口，不是最快的，但可能是最诚实的那个。
                </p>
                <p>
                  这本书不教你&quot;5 分钟上手&quot;，而是带你理解底层原理。因为只有理解原理，才能在出问题时知道修哪里。
                </p>
              </div>
            </div>
          </div>
        </section>
      </main>

      <BlogFooter />
    </div>
  );
}

function Stat({ value, label }: { value: string; label: string }) {
  return (
    <div className="flex items-baseline gap-1">
      <span className="text-3xl font-bold text-primary-text">{value}</span>
      {label && <span className="text-sm text-secondary-text">{label}</span>}
    </div>
  );
}
