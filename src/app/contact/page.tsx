import type { Metadata } from "next";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  title: "Contact",
  description: "Contact Pulseclaw for product conversations, research, collaboration, media, or investment.",
};

const contactChannels = [
  {
    label: "Email",
    value: siteConfig.email,
    href: siteConfig.links.email,
    note: "最直接的联系入口，适合合作、产品沟通、媒体与投资交流。",
  },
  {
    label: "GitHub",
    value: "Pulseclaw Repository",
    href: siteConfig.links.github,
    note: "查看项目代码、公开仓库与当前产品实现。",
  },
  {
    label: "X",
    value: "@wjnhng419090",
    href: siteConfig.links.x,
    note: "适合公开动态、产品进展和简短沟通。",
  },
  {
    label: "公众号文章",
    value: "产品思考与公开内容",
    href: siteConfig.links.wechat,
    note: "当前的中文公开内容入口，适合了解产品想法与方向。",
  },
];

export default function ContactPage() {
  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-14 pt-12 sm:pb-18 lg:pb-20 lg:pt-18">
          <Container>
            <div className="grid gap-10 lg:grid-cols-[0.9fr_1.1fr] lg:items-end">
              <div className="max-w-[38rem]">
                <span className="section-kicker">Contact</span>
                <h1 className="mt-6 text-[clamp(2.8rem,6vw,4.7rem)] font-semibold leading-[0.98] text-primary-text">
                  对外沟通的正式入口，
                  <span className="mt-3 block text-mist-blue">现在已经可以稳定公开使用。</span>
                </h1>
                <p className="mt-6 text-[1.05rem] leading-8 text-secondary-text sm:text-[1.12rem]">
                  如果你想聊产品、研究、合作、媒体或投资相关内容，这一页就是最直接的对外入口。
                </p>
              </div>

              <div className="surface-panel rounded-[1.9rem] p-5 sm:p-6">
                <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                  Primary contact
                </div>
                <div className="mt-3 text-[1.45rem] font-semibold text-primary-text">{siteConfig.email}</div>
                <p className="mt-3 text-sm leading-7 text-secondary-text">
                  当前最适合的正式联系渠道是邮件。网站与公开页面上的所有联络入口，都会收束到这里。
                </p>
                <a
                  href={siteConfig.links.email}
                  className="mt-5 inline-flex rounded-full bg-primary-text px-5 py-2.5 text-sm font-semibold text-white shadow-[0_16px_32px_rgba(45,42,38,0.16)] transition-colors hover:bg-primary-text/94"
                >
                  Send email
                </a>
              </div>
            </div>
          </Container>
        </section>

        <section className="pb-16 sm:pb-20">
          <Container>
            <div className="grid gap-4 lg:grid-cols-2">
              {contactChannels.map((channel) => (
                <a
                  key={channel.label}
                  href={channel.href}
                  target={channel.href.startsWith("mailto:") ? undefined : "_blank"}
                  rel={channel.href.startsWith("mailto:") ? undefined : "noreferrer"}
                  className="surface-panel rounded-[1.75rem] p-5 transition-transform duration-300 hover:-translate-y-0.5"
                >
                  <div className="text-[11px] font-semibold uppercase tracking-[0.18em] text-secondary-text/66">
                    {channel.label}
                  </div>
                  <div className="mt-3 text-[1.3rem] font-semibold text-primary-text">{channel.value}</div>
                  <p className="mt-3 text-sm leading-7 text-secondary-text">{channel.note}</p>
                </a>
              ))}
            </div>
          </Container>
        </section>

        <section className="pb-20 sm:pb-24">
          <Container>
            <div className="surface-panel-strong rounded-[2rem] p-6 sm:p-8">
              <div className="grid gap-6 lg:grid-cols-[0.85fr_1.15fr] lg:items-start">
                <div>
                  <span className="section-kicker">Good for</span>
                  <h2 className="mt-5 text-[clamp(2rem,4vw,3rem)] font-semibold leading-[1.05] text-primary-text">
                    现在这套站点已经适合被公开发出去了。
                  </h2>
                </div>

                <div className="grid gap-3 sm:grid-cols-2">
                  {[
                    "产品介绍与首次认识",
                    "公开演示与产品 proof",
                    "博客与产品思考沉淀",
                    "合作、媒体与投资沟通",
                  ].map((item) => (
                    <div
                      key={item}
                      className="rounded-[1.35rem] border border-black/6 bg-white/84 px-4 py-4 text-sm font-medium text-primary-text"
                    >
                      {item}
                    </div>
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
