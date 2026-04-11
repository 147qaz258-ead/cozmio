import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { getAllPosts, getPostBySlug } from "@/lib/blog";

export function generateStaticParams() {
  return getAllPosts().map((post) => ({ slug: post.slug }));
}

export async function generateMetadata({ params }: { params: Promise<{ slug: string }> }): Promise<Metadata> {
  const { slug } = await params;
  const post = getPostBySlug(slug);

  if (!post) {
    return { title: "Blog" };
  }

  return {
    title: post.title,
    description: post.summary,
    alternates: {
      canonical: `https://cozmio.net/blog/${slug}`,
    },
    openGraph: {
      type: "article",
      url: `https://cozmio.net/blog/${slug}`,
      publishedTime: post.publishedAt,
      modifiedTime: post.publishedAt,
      tags: post.tags,
    },
    twitter: {
      card: "summary_large_image",
    },
  };
}

export default async function BlogPostPage({ params }: { params: Promise<{ slug: string }> }) {
  const { slug } = await params;
  const post = getPostBySlug(slug);

  if (!post) {
    notFound();
  }

  const relatedPosts = getAllPosts().filter((entry) => entry.slug !== post.slug).slice(0, 2);

  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-14 pt-12 sm:pb-18 lg:pb-20 lg:pt-18">
          <Container>
            <div className="max-w-[50rem]">
              <Link
                href="/blog"
                className="inline-flex rounded-full border border-black/6 bg-white/74 px-4 py-2 text-sm font-medium text-secondary-text transition-colors hover:bg-white hover:text-primary-text"
              >
                返回 Blog
              </Link>
              <div className="mt-6 text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                {post.kicker}
              </div>
              <h1 className="mt-4 text-[clamp(2.7rem,5vw,4.4rem)] font-semibold leading-[0.98] text-primary-text">
                {post.title}
              </h1>
              <p className="mt-6 text-[1.02rem] leading-8 text-secondary-text sm:text-[1.08rem]">{post.intro}</p>

              <div className="mt-6 flex flex-wrap items-center gap-3 text-sm text-secondary-text">
                <span>{post.category}</span>
                <span>·</span>
                <span>{post.readingTime}</span>
                <span>·</span>
                <span>{post.publishedAt}</span>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-16 sm:pb-20">
          <Container>
            <div className="grid gap-8 lg:grid-cols-[1.08fr_0.92fr]">
              <article className="surface-panel-strong rounded-[2rem] p-6 sm:p-8">
                <div className="space-y-8">
                  {post.sections.map((section) => (
                    <section key={section.heading} className="rounded-[1.6rem] border border-black/6 bg-white/82 p-5">
                      <h2 className="text-[1.4rem] font-semibold text-primary-text">{section.heading}</h2>
                      <div className="mt-4 space-y-3 text-sm leading-7 text-secondary-text">
                        {section.paragraphs.map((paragraph) => (
                          <p key={paragraph}>{paragraph}</p>
                        ))}
                      </div>
                    </section>
                  ))}
                </div>
              </article>

              <aside className="space-y-5">
                <div className="surface-panel rounded-[1.8rem] p-5">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                    Tags
                  </div>
                  <div className="mt-4 flex flex-wrap gap-2">
                    {post.tags.map((tag) => (
                      <span
                        key={tag}
                        className="rounded-full border border-black/6 bg-white/76 px-3 py-1.5 text-[11px] font-semibold uppercase tracking-[0.14em] text-secondary-text/74"
                      >
                        {tag}
                      </span>
                    ))}
                  </div>
                </div>

                <div className="surface-panel rounded-[1.8rem] p-5">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                    Related writing
                  </div>
                  <div className="mt-4 space-y-3">
                    {relatedPosts.map((entry) => (
                      <Link
                        key={entry.slug}
                        href={`/blog/${entry.slug}`}
                        className="block rounded-[1.35rem] border border-black/6 bg-white/82 px-4 py-4 transition-colors hover:bg-white"
                      >
                        <div className="text-sm font-semibold text-primary-text">{entry.title}</div>
                        <div className="mt-2 text-sm leading-7 text-secondary-text">{entry.summary}</div>
                      </Link>
                    ))}
                  </div>
                </div>
              </aside>
            </div>
          </Container>
        </section>
      </main>

      <Footer />
    </div>
  );
}
