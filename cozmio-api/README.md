# Cozmio API Service

后端 API 服务，提供数据存储和业务逻辑处理。

## 技术栈

- Go
- PostgreSQL
- Drizzle ORM

## 环境变量

复制 `.env.example` 为 `.env` 并配置：

```
DATABASE_URL=postgresql://user:password@localhost:5432/cozmio_db
PORT=3000
```

## 启动

```bash
go build
./cozmio-api
```

或使用 Docker：

```bash
docker-compose up
```