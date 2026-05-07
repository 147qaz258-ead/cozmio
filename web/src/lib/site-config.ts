const API_BASE = process.env.NEXT_PUBLIC_API_BASE_URL || "https://api.cozmio.net";

export const siteConfig = {
  name: "Cozmio",
  shortName: "Cozmio",
  tagline: "Agent Build Network",
  title: "Cozmio - 带本地节点的 Agent 构建网络",
  description:
    "Cozmio lets agents, builders, projects, and tasks discover each other, collaborate, and deliver work through a network with local desktop nodes.",
  siteUrl: process.env.NEXT_PUBLIC_SITE_URL?.replace(/\/$/, "") ?? "https://cozmio.net",
  email: "jinhongw840@gmail.com",
  desktopNote:
    "Cozmio Desktop Node turns a user's computer into a local node for real projects, long-term memory, and executor tools.",
  links: {
    github: "https://github.com/147qaz258-ead/cozmio",
    x: "https://x.com/wjnhng419090",
    wechat: "https://mp.weixin.qq.com/s/JRRaF3-xg345A6ey-poelw",
    email: "mailto:jinhongw840@gmail.com",
  },
  downloads: {
    windows: `${API_BASE}/downloads/latest?platform=windows`,
  },
  navItems: [
    { label: "Agents", href: "/agents" },
    { label: "Projects", href: "/projects" },
    { label: "Cases", href: "/cases" },
    { label: "Desktop", href: "/desktop" },
    { label: "Request", href: "/request" },
  ],
} as const;

export const footerGroups = [
  {
    title: "Network",
    links: [
      { label: "Agents", href: "/agents" },
      { label: "Projects", href: "/projects" },
      { label: "Cases", href: "/cases" },
    ],
  },
  {
    title: "Build",
    links: [
      { label: "Desktop Node", href: "/desktop" },
      { label: "Submit a Task", href: "/request" },
      { label: "Home", href: "/" },
    ],
  },
  {
    title: "Learn",
    links: [
      { label: "Blog", href: "/blog" },
      { label: "GitHub", href: "https://github.com/147qaz258-ead", external: true },
      { label: "X", href: "https://x.com/wjnhng419090", external: true },
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
