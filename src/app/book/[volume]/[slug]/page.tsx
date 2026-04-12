import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { BlogHeader } from "@/components/blog/BlogHeader";
import { BlogFooter } from "@/components/blog/BlogFooter";
import { MarkdownRenderer } from "@/components/MarkdownRenderer";
import { TocSidebar } from "@/components/TocSidebar";
import { ChapterNavigation } from "@/components/ChapterNavigation";
import { getAllVolumes, getChapterBySlug, getAdjacentChapters } from "@/lib/book";

export async function generateStaticParams() {
  const volumes = getAllVolumes();
  const params: { volume: string; slug: string }[] = [];

  volumes.forEach((vol) => {
    vol.chapters.forEach((ch) => {
      params.push({ volume: vol.id, slug: ch.slug });
    });
  });

  return params;
}

export async function generateMetadata({
  params,
}: {
  params: Promise<{ volume: string; slug: string }>;
}): Promise<Metadata> {
  const { volume, slug } = await params;
  const chapter = await getChapterBySlug(volume, slug);

  if (!chapter) {
    return { title: "OpenClaw Book" };
  }

  return {
    title: `${chapter.title} — OpenClaw Book`,
    description: chapter.excerpt,
  };
}

export default async function ChapterPage({
  params,
}: {
  params: Promise<{ volume: string; slug: string }>;
}) {
  const { volume, slug } = await params;
  const chapter = await getChapterBySlug(volume, slug);

  if (!chapter) {
    notFound();
  }

  const adjacent = await getAdjacentChapters(volume, slug);

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <BlogHeader />

      <main className="flex-1">
        {/* Article Header */}
        <section className="border-b border-black/6 py-12 lg:py-16">
          <div className="mx-auto max-w-6xl px-6">
            <div className="max-w-3xl">
              {/* Breadcrumb */}
              <div className="mb-6 flex items-center gap-2 text-sm text-secondary-text">
                <Link href="/blog" className="hover:text-mist-blue transition-colors">
                  全部章节
                </Link>
                <span>/</span>
                <span>{chapter.volume}</span>
              </div>

              {/* Chapter Badge */}
              <div className="mb-4 inline-flex items-center gap-2 rounded-full border border-mist-blue/20 bg-mist-blue/5 px-3 py-1">
                <span className="text-xs font-medium text-mist-blue">第 {chapter.chapterIndex} 章</span>
              </div>

              {/* Title */}
              <h1 className="text-[clamp(2rem,4vw,3rem)] font-bold leading-tight text-primary-text">
                {chapter.title}
              </h1>

              {/* Meta */}
              <div className="mt-6 flex flex-wrap items-center gap-4 text-sm text-secondary-text">
                <span>{chapter.words || "约 15000 字"}</span>
                <span className="h-1 w-1 rounded-full bg-secondary-text/30" />
                <span>{chapter.status === "draft" ? "Draft" : "Published"}</span>
                <span className="h-1 w-1 rounded-full bg-secondary-text/30" />
                <span>OpenClaw Book</span>
              </div>
            </div>
          </div>
        </section>

        {/* Article Content */}
        <section className="py-12 lg:py-16">
          <div className="mx-auto max-w-6xl px-6">
            <div className="grid gap-12 lg:grid-cols-[1fr_280px]">
              {/* Main Content */}
              <article className="min-w-0">
                <div className="prose prose-lg max-w-none prose-headings:font-bold prose-headings:text-primary-text prose-p:text-secondary-text prose-p:leading-8 prose-a:text-mist-blue prose-a:no-underline prose-code:text-primary-text prose-code:bg-black/5 prose-code:px-1.5 prose-code:py-0.5 prose-code:rounded prose-pre:bg-zinc-900 prose-pre:rounded-xl">
                  <MarkdownRenderer content={chapter.contentHtml} />
                </div>

                {/* Chapter Navigation */}
                <div className="mt-16 pt-8 border-t border-black/6">
                  <ChapterNavigation
                    volumeId={volume}
                    prev={adjacent.prev}
                    next={adjacent.next}
                  />
                </div>
              </article>

              {/* TOC Sidebar */}
              <aside className="hidden lg:block">
                <div className="sticky top-24">
                  <div className="text-xs font-semibold uppercase tracking-widest text-secondary-text/50 mb-4">
                    目录
                  </div>
                  <TocSidebar items={chapter.toc} />
                </div>
              </aside>
            </div>
          </div>
        </section>
      </main>

      <BlogFooter />
    </div>
  );
}