# 服务器信息

## VPS
- IP: 47.76.116.209
- OS: Ubuntu 22.04.5 LTS
- Hostname: iZj6c652cfy7a46w7rrmchZ
- Provider: 阿里云（Alibaba Cloud）香港节点
- SSH 已配置密码登录
- Password: WuJinHong132

## SSH 连接
```bash
ssh root@47.76.116.209
# 密码: WuJinHong132
```

## 部署需求
1. cozmio-api: Node.js 后端 (Fastify + PostgreSQL + Drizzle)
   - 需要 PostgreSQL 16 数据库
   - Docker Compose 部署
   - 端口: 3001
2. cozmio/web: Next.js 16 前端
   - 静态导出，部署到 Cloudflare Pages
   - 构建产物 out/ 目录

## Docker 环境验证（待执行）
- Docker 版本
- Docker Compose 版本
- 当前运行中的容器