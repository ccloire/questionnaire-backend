# 问卷星后端

基于Rust和Axum框架构建的问卷系统后端。

## 项目结构

```
questionnaire-backend/
├── src/
│   ├── config/              # 配置相关
│   ├── models/              # 数据模型
│   ├── routes/              # 路由处理
│   ├── services/            # 业务逻辑
│   ├── utils/               # 工具函数
│   └── main.rs              # 入口文件
├── database/                # 数据库相关脚本
│   ├── init.sql             # 初始化数据库结构
│   └── sample_data.sql      # 示例数据
├── Cargo.toml               # 项目依赖
├── .env.example             # 环境变量示例
└── README.md                # 项目说明
```

## 功能特性

- 用户认证：注册、登录、JWT认证
- 问卷管理：创建、编辑、删除、查询问卷
- 问卷回答：提交问卷回答、查看回答统计
- RESTful API设计
- 统一的错误处理和响应格式

## 技术栈

- Rust
- Axum Web框架
- MySQL数据库 (使用SQLx)
- JWT认证
- Serde序列化/反序列化
- Tokio异步运行时

## 开始使用

### 环境要求

- Rust (推荐1.75.0或更高版本)
- MySQL 5.7或更高版本
- 对应前端项目: Vue.js问卷前端

### 配置

1. 复制环境变量示例文件并修改为你的配置:

```bash
cp .env.example .env
```

2. 修改`.env`文件中的配置，特别是数据库连接和JWT密钥:

```
DATABASE_URL=mysql://用户名:密码@localhost:3306/questionnaire
JWT_SECRET=你的密钥
```

### 数据库初始化

1. 创建数据库和表结构:

```bash
mysql -u root -p < database/init.sql
```

2. (可选) 导入示例数据:

```bash
mysql -u root -p < database/sample_data.sql
```

### 运行

1. 编译和运行项目:

```bash
cargo run
```

2. 服务将在配置的端口上启动 (默认为3000)

## API接口

### 用户相关

- `POST /api/users/register` - 用户注册
- `POST /api/users/login` - 用户登录
- `GET /api/users/me` - 获取当前用户信息 (需认证)

### 问卷相关

- `GET /api/questionnaires/public` - 获取公开问卷列表
- `GET /api/questionnaires/:id` - 获取问卷详情
- `GET /api/questionnaires/my` - 获取我的问卷列表 (需认证)
- `POST /api/questionnaires` - 创建问卷 (需认证)
- `PUT /api/questionnaires/:id` - 更新问卷 (需认证)
- `DELETE /api/questionnaires/:id` - 删除问卷 (需认证)

### 问卷回答相关

- `POST /api/responses/submit` - 提交问卷回答
- `GET /api/responses/questionnaires/:id/statistics` - 获取问卷统计信息 (需认证)
- `GET /api/responses/questionnaires/:id/responses` - 获取问卷回答列表 (需认证)
- `GET /api/responses/:id` - 获取回答详情 (需认证)

## 前后端通信

前端通过axios库发送HTTP请求与后端通信。主要流程如下:

1. 用户登录后，前端将JWT令牌存储在localStorage中
2. 对于需要认证的请求，前端会自动在请求头中添加Authorization Bearer令牌
3. 数据交换采用JSON格式
4. 所有API响应遵循统一格式:
   ```json
   {
     "code": 200,
     "success": true,
     "message": "操作成功",
     "data": { ... }
   }
   ```

## 与前端集成

要与Vue.js前端集成:

1. 确保后端服务已启动并监听配置的端口
2. 前端项目的开发服务器配置了API代理，将`/api`请求代理到后端服务
3. 启动前端开发服务器

前端项目的proxy配置示例 (vite.config.ts):
```typescript
export default defineConfig({
  // ...其他配置
  server: {
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true
      }
    }
  }
})
```

## 许可

MIT 