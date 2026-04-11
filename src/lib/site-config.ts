export const siteConfig = {
  name: "Pulseclaw",
  shortName: "Pulseclaw",
  tagline: "Context Before Prompt",
  title: "Pulseclaw - 先保留上下文，再让 AI 开口",
  description:
    "Pulseclaw is a desktop-first, local-first product for replayable, evidence-first context capture and bounded AI help.",
  siteUrl: process.env.NEXT_PUBLIC_SITE_URL?.replace(/\/$/, "") ?? "https://cozmio.net",
  email: "jinhongw840@gmail.com",
  desktopNote:
    "Pulseclaw 是桌面端产品。这一网站是它的正式官网、公开演示入口与思考输出窗口。",
  links: {
    github: "https://github.com/147qaz258-ead/Pulseclaw",
    x: "https://x.com/wjnhng419090",
    wechat: "https://mp.weixin.qq.com/s/JRRaF3-xg345A6ey-poelw",
    email: "mailto:jinhongw840@gmail.com",
  },
  navItems: [
    { label: "Product", href: "/" },
    { label: "Demo", href: "/demo" },
    { label: "Progress", href: "/progress" },
    { label: "Proof", href: "/demo/debug-bug" },
    { label: "Blog", href: "/blog" },
    { label: "About", href: "/about" },
    { label: "Contact", href: "/contact" },
  ],
} as const;

export const footerGroups = [
  {
    title: "Product",
    links: [
      { label: "Home", href: "/" },
      { label: "Demo Hub", href: "/demo" },
      { label: "Pulseclaw", href: "/progress" },
      { label: "验证演示", href: "/demo/debug-bug" },
    ],
  },
  {
    title: "Writing",
    links: [
      { label: "Blog", href: "/blog" },
      { label: "About", href: "/about" },
      { label: "Contact", href: "/contact" },
    ],
  },
  {
    title: "Links",
    links: [
      { label: "GitHub", href: "https://github.com/147qaz258-ead/Pulseclaw", external: true },
      { label: "X", href: "https://x.com/wjnhng419090", external: true },
      {
        label: "公众号文章",
        href: "https://mp.weixin.qq.com/s/JRRaF3-xg345A6ey-poelw",
        external: true,
      },
    ],
  },
  {
    title: "Legal",
    links: [
      { label: "Privacy Policy", href: "/privacy" },
      { label: "Terms of Service", href: "/terms" },
      { label: "Email", href: "mailto:jinhongw840@gmail.com", external: true },
    ],
  },
] as const;
