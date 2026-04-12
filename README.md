# Pulseclaw 官网

> 先保留你刚经历过的上下文，再让 AI 开口。

[pulseclaw.cozmio.net](https://pulseclaw.cozmio.net) 是 Pulseclaw 的正式官网、公开演示与产品思考入口。

## 产品定位

Pulseclaw 是桌面端工具——自动捕获工作上下文，在证据支撑下提供 AI 帮助。

传统的 AI 交互要求用户先把一切翻译成 prompt。Pulseclaw 的思路不同：**先把真实的工作痕迹接住，再在证据边界内给出候选帮助**。

核心特性：
- **上下文先于提示词** — 工作现场的原始片段被直接接入系统，不需要用户主动描述
- **证据支撑** — 每条帮助都可以回指到原始证据链，可验证、可重新组织
- **本地优先** — 上下文捕获优先停留在本地，系统靠近现场本身而不是先变成远端摘要
- **候选帮助，有边界** — 帮助以候选形式出现，系统不声称理解全部目标

## 技术栈

- **Next.js 16** (App Router, 静态导出)
- **TypeScript**
- **Tailwind CSS** + 自定义设计系统
- **Cloudflare Pages** (部署)

## 快速上手

```bash
git clone https://github.com/147qaz258-ead/cozmio.git
cd cozmio
npm install
npm run dev
```

访问 [http://localhost:3000](http://localhost:3000)

## 目录结构

```
src/
├── app/                    # Next.js App Router 页面
│   ├── page.tsx            # 首页
│   ├── demo/               # 演示中心
│   ├── blog/               # 产品思考
│   ├── progress/           # 迭代进度（git 构建历史）
│   └── about/              # 关于
├── components/             # React 组件
│   ├── layout/             # Header / Footer
│   └── demo/               # Demo 可视化组件
└── lib/                    # 数据层和工具函数
    ├── blog.ts              # 博客内容
    ├── progress-data.ts     # Git 构建历史读取
    └── site-config.ts      # 站点配置
```

## 部署

push 到 `master` 分支后，GitHub Actions 自动构建并部署到 Cloudflare Pages。

手动部署：

```bash
npm run deploy:site        # 构建 + 部署
npm run deploy:site:fast   # 跳过 lint，直接部署
```

## 相关链接

- 产品官网：https://pulseclaw.cozmio.net
- GitHub：https://github.com/147qaz258-ead/pulseclaw（桌面端产品仓库）
