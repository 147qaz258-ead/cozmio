import type { Metadata } from "next";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  title: "Terms of Service",
  description: "Terms of Service for the Pulseclaw public website.",
};

const sections = [
  {
    title: "Use of the Website",
    body: [
      "Pulseclaw 当前网站用于公开展示产品定位、演示内容、博客文章与对外联系信息。访问本网站，即表示你同意以合法、合规的方式使用这些公开内容。",
      "当前网站并不承诺提供一个已经商业开放的 web app。Pulseclaw 的产品方向是桌面端，这一网站主要承担正式官网与公开展示的职责。",
    ],
  },
  {
    title: "Intellectual Property",
    body: [
      "除外部平台内容、第三方商标或另有说明的材料外，Pulseclaw 网站中的品牌表达、文案、演示内容与站点设计受适用的知识产权规则保护。",
      "你可以分享、引用和讨论公开页面内容，但不应误导性地声称自己拥有或代表 Pulseclaw 的官方身份。",
    ],
  },
  {
    title: "Demo and Product Information",
    body: [
      "网站中的 Demo 与 Proof 页面用于展示当前产品方向、交互表达与系统边界。它们不等同于一项已经对外提供完整商业服务的在线平台承诺。",
      "Pulseclaw 会尽量保持公开内容准确，但不会把展示性内容表述成超出当前阶段的正式保证。",
    ],
  },
  {
    title: "External Services",
    body: [
      "网站可能链接到 GitHub、X、微信公众号文章或其他第三方平台。访问这些外部资源时，你需要自行遵守对应平台的规则和政策。",
      "Pulseclaw 不对第三方平台的可用性、内容准确性或后续变更承担超出法律要求范围的责任。",
    ],
  },
  {
    title: "Contact",
    body: [
      `如需就本条款联系，请发送邮件至 ${siteConfig.email}。`,
      "本条款会随着网站与产品公开形态的变化进行更新。",
    ],
  },
];

export default function TermsPage() {
  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-14 pt-12 sm:pb-18 lg:pb-20 lg:pt-18">
          <Container>
            <div className="max-w-[48rem]">
              <span className="section-kicker">Terms of Service</span>
              <h1 className="mt-6 text-[clamp(2.7rem,5vw,4.4rem)] font-semibold leading-[0.98] text-primary-text">
                Pulseclaw 网站服务条款
              </h1>
              <p className="mt-6 text-[1.02rem] leading-8 text-secondary-text sm:text-[1.08rem]">
                最后更新于 2026 年 4 月 4 日。本页覆盖的是 Pulseclaw 公开网站与公开展示内容的使用条款。
              </p>
            </div>
          </Container>
        </section>

        <section className="pb-20 sm:pb-24">
          <Container>
            <div className="surface-panel-strong rounded-[2rem] p-6 sm:p-8 lg:p-10">
              <div className="space-y-8">
                {sections.map((section) => (
                  <section key={section.title} className="rounded-[1.6rem] border border-black/6 bg-white/82 p-5">
                    <h2 className="text-[1.35rem] font-semibold text-primary-text">{section.title}</h2>
                    <div className="mt-4 space-y-3 text-sm leading-7 text-secondary-text">
                      {section.body.map((paragraph) => (
                        <p key={paragraph}>{paragraph}</p>
                      ))}
                    </div>
                  </section>
                ))}
              </div>
            </div>
          </Container>
        </section>
      </main>

      <Footer />
    </div>
  );
}
