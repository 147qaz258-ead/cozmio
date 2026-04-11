import type { Metadata } from "next";
import Link from "next/link";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  title: "About",
  description:
    "Learn what Pulseclaw is building and why it starts with desktop-first, replayable, evidence-first context capture.",
};

const principles = [
  {
    title: "Desktop-first",
    description:
      "Pulseclaw 的产品形态首先面向桌面工作现场，而不是假装所有上下文都发生在一个网页输入框里。",
  },
  {
    title: "Evidence-first",
    description:
      "原始记录、回放能力和边界清晰的候选层，一起构成这套系统的可信度基础。",
  },
  {
    title: "Local-first",
    description:
      "上下文捕获优先停留在本地，系统尽量靠近现场本身，而不是先把一切变成远端摘要。",
  },
];

const externalLinks = [
  { label: "GitHub", href: siteConfig.links.github },
  { label: "X", href: siteConfig.links.x },
  { label: "公众号文章", href: siteConfig.links.wechat },
  { label: "Contact", href: siteConfig.links.email },
];

export default function AboutPage() {
  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-14 pt-12 sm:pb-18 lg:pb-20 lg:pt-18">
          <Container>
            <div className="grid gap-10 lg:grid-cols-[0.92fr_1.08fr] lg:items-end">
              <div className="max-w-[40rem]">
                <span className="section-kicker">About Pulseclaw</span>
                <h1 className="mt-6 text-[clamp(2.8rem,6vw,4.8rem)] font-semibold leading-[0.98] text-primary-text">
                  这是一个桌面端产品官网，
                  <span className="mt-3 block text-mist-blue">也是公开演示与思考输出的正式入口。</span>
                </h1>
                <p className="mt-6 text-[1.05rem] leading-8 text-secondary-text sm:text-[1.12rem]">
                  Pulseclaw 关注的是一个更基础的问题：为什么用户已经在工作里表达了很多东西，
                  AI 却总要等一句 prompt 才开始理解。我们的答案不是更早抢答，而是先把真实上下文接住。
                </p>
              </div>

              <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                  Site role
                </div>
                <p className="mt-3 text-sm leading-7 text-secondary-text">{siteConfig.desktopNote}</p>

                <div className="mt-5 grid gap-3 sm:grid-cols-2">
                  <Link
                    href="/demo"
                    className="rounded-[1.3rem] border border-black/6 bg-white/82 px-4 py-4 text-sm font-medium text-primary-text transition-colors hover:bg-white"
                  >
                    打开 Demo Hub
                  </Link>
                  <Link
                    href="/blog"
                    className="rounded-[1.3rem] border border-black/6 bg-white/82 px-4 py-4 text-sm font-medium text-primary-text transition-colors hover:bg-white"
                  >
                    阅读 Blog
                  </Link>
                </div>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-16 sm:pb-20">
          <Container>
            <div className="grid gap-5 lg:grid-cols-3">
              {principles.map((item) => (
                <article key={item.title} className="surface-panel rounded-[1.75rem] p-5">
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                    Principle
                  </div>
                  <h2 className="mt-3 text-[1.35rem] font-semibold text-primary-text">{item.title}</h2>
                  <p className="mt-3 text-sm leading-7 text-secondary-text">{item.description}</p>
                </article>
              ))}
            </div>
          </Container>
        </section>

        <section className="pb-16 sm:pb-20">
          <Container>
            <div className="surface-panel-strong rounded-[2rem] p-6 sm:p-8">
              <div className="grid gap-8 lg:grid-cols-[0.85fr_1.15fr]">
                <div className="max-w-[30rem]">
                  <span className="section-kicker">Why Now</span>
                  <h2 className="mt-5 text-[clamp(2rem,4vw,3rem)] font-semibold leading-[1.05] text-primary-text">
                    Pulseclaw 现在优先把“上下文进入系统”这一步先做扎实。
                  </h2>
                  <p className="mt-4 text-[1rem] leading-7 text-secondary-text">
                    首页、Demo、Proof 与 Blog 共同组成了当前站点的公开表达。它们一起说明产品在解决什么、
                    为什么要这样设计，以及为什么边界比夸张承诺更重要。
                  </p>
                </div>

                <div className="grid gap-4">
                  {[
                    "首页负责产品定位与总叙事。",
                    "Demo Hub 负责展示真实片段如何长成输出。",
                    "验证演示页负责证明原始记录、可回放与候选层边界。",
                    "Blog 负责把产品思考公开沉淀出来。",
                  ].map((line) => (
                    <div
                      key={line}
                      className="rounded-[1.4rem] border border-black/6 bg-white/84 px-5 py-4 text-sm leading-7 text-primary-text"
                    >
                      {line}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-20 sm:pb-24">
          <Container>
            <div className="grid gap-5 lg:grid-cols-[0.88fr_1.12fr]">
              <div className="surface-panel rounded-[1.8rem] p-5 sm:p-6">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                  Contact
                </div>
                <h2 className="mt-3 text-[1.5rem] font-semibold text-primary-text">对外联系</h2>
                <p className="mt-3 text-sm leading-7 text-secondary-text">
                  关于产品交流、合作、媒体、研究或投资沟通，可以直接通过邮件联系。
                </p>
                <a
                  href={siteConfig.links.email}
                  className="mt-5 inline-flex rounded-full bg-primary-text px-4 py-2.5 text-sm font-semibold text-white shadow-[0_16px_32px_rgba(45,42,38,0.16)] transition-colors hover:bg-primary-text/94"
                >
                  {siteConfig.email}
                </a>
              </div>

              <div className="surface-panel rounded-[1.8rem] p-5 sm:p-6">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                  External links
                </div>
                <div className="mt-5 grid gap-3 sm:grid-cols-2">
                  {externalLinks.map((link) => (
                    <a
                      key={link.label}
                      href={link.href}
                      target="_blank"
                      rel="noreferrer"
                      className="rounded-[1.35rem] border border-black/6 bg-white/82 px-4 py-4 text-sm font-medium text-primary-text transition-colors hover:bg-white"
                    >
                      {link.label}
                    </a>
                  ))}
                </div>
              </div>
            </div>
          </Container>
        </section>
      </main>

      <Footer />
    </div>
  );
}
