# Cozmio Web

Cozmio 项目的官方网站前端。

## 页面

- `/` - 首页
- `/about` - 关于
- `/desktop` - 桌面应用介绍
- `/use` - 使用页面（含任务管理）
- `/agents` - 智能体介绍
- `/cases` - 案例展示
- `/blog` - 博客
- `/request` - 申请使用

## 开发

```bash
npm install
npm run dev
```

## 构建

```bash
NEXT_PUBLIC_API_BASE_URL=/api npm run build
```

构建产物在 `out/` 目录，可部署到任意静态托管服务。

## 技术栈

- Next.js 16
- React 19
- TypeScript
- Framer Motion
- Tailwind CSS