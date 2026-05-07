import type { Metadata } from "next";
import { Header } from "@/components/layout/Header";
import { Footer } from "@/components/layout/Footer";
import { Container } from "@/components/layout/Container";
import { siteConfig } from "@/lib/site-config";

export const metadata: Metadata = {
  title: "Privacy Policy",
  description: "Privacy Policy for the Pulseclaw public website.",
};

const sections = [
  {
    title: "Overview",
    body: [
      "Pulseclaw 当前是桌面产品的公开官网、演示入口与博客站点。网站本身不要求注册账号，也不要求提交私有工作内容才能浏览公开页面。",
      "本页说明的是当前公开网站层面的隐私处理方式，不把尚未公开提供的桌面产品能力包装成已经上线的网络服务。",
    ],
  },
  {
    title: "Information You Voluntarily Provide",
    body: [
      "如果你通过邮件、GitHub、X 或其他外部渠道主动联系 Pulseclaw，相关平台可能会处理你主动提供的信息，例如邮箱地址、消息正文或公开账号资料。",
      "网站本身目前没有内建联系表单，因此不会在站内额外创建一套新的提交数据库来收集这些内容。",
    ],
  },
  {
    title: "Website Data",
    body: [
      "公开网站当前主要提供静态页面、演示页面与博客内容。站点本身不要求登录，也不设计为上传你的本地工作上下文。",
      "和大多数网站一样，托管平台、CDN 或服务器基础设施可能会产生基础访问日志、错误日志与性能数据，这些属于标准网站运维范围。",
    ],
  },
  {
    title: "External Links",
    body: [
      "网站包含通往 GitHub、X、微信公众号文章与其他外部页面的链接。点击这些链接后，你将进入第三方平台，并受对应平台的隐私政策与使用规则约束。",
      "Pulseclaw 不控制这些第三方平台，也不对它们各自的数据处理方式做超出公开信息范围的承诺。",
    ],
  },
  {
    title: "Contact",
    body: [
      `如果你对本页有疑问，可以通过 ${siteConfig.email} 联系。`,
      "本政策会随着网站内容与公开产品形态的变化进行更新。",
    ],
  },
];

export default function PrivacyPage() {
  return (
    <div className="flex min-h-screen flex-col bg-warm-white">
      <Header />

      <main className="flex-1">
        <section className="pb-14 pt-12 sm:pb-18 lg:pb-20 lg:pt-18">
          <Container>
            <div className="max-w-[48rem]">
              <span className="section-kicker">Privacy Policy</span>
              <h1 className="mt-6 text-[clamp(2.7rem,5vw,4.4rem)] font-semibold leading-[0.98] text-primary-text">
                Pulseclaw 网站隐私政策
              </h1>
              <p className="mt-6 text-[1.02rem] leading-8 text-secondary-text sm:text-[1.08rem]">
                最后更新于 2026 年 4 月 4 日。当前政策覆盖的是公开网站层面的使用与访问，不把尚未开放的产品能力写成已经提供的在线服务。
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
