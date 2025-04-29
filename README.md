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
如果您将后端托管到云服务器而不是本地运行，您需要修改配置如下：
- 对于IP地址：
  - 将SERVER_HOST改为0.0.0.0以允许从任何IP访问，或设置为您云服务器的具体IP
  - 例如：SERVER_HOST=0.0.0.0
- 对于端口：
  - 可以保留SERVER_PORT=8080或根据云服务器的要求更改
  - 确保该端口在云服务器的防火墙中是开放的
- 其他可能需要更改的配置：
  - 数据库连接信息（如果数据库也在云服务器上）
  - 跨域设置（CORS）以允许前端访问
  - 日志级别和存储位置
  - SSL/TLS证书配置（如果需要HTTPS）
- 部署注意事项：
  - 确保创建相应的配置文件（如config.toml或.env）并放在云服务器上
  - 设置适当的环境变量
  - 考虑使用环境变量而不是配置文件进行敏感设置
记得在云服务器上适当配置防火墙，只开放必要的端口，并考虑使用反向代理（如Nginx）来增强安全性和性能。

## .gitignore说明
项目在后端服务器（本项目暂时只有开发者本地运行环境）运行后会生成Cargo.lock和target文件夹这些缓存文件，这些并不是项目本身原始文件，因此被添加为忽略项。使用者本地运行后自然会生成。  
.env是环境变量配置文件，用于指定后端服务器端口和数据库端口，JWT密钥配置以及日志配置。这些需要套用该后端时自行创建.env进行配置，src/main.rs里会对该文件进行调用，以与前端端口和进行CORS跨域通信，并连接你的数据库。  
下面是一个示例：
```
# 服务器配置
SERVER_HOST=127.0.0.1
SERVER_PORT=8080

# 数据库配置
DATABASE_URL=mysql://用户名:密码@localhost:数据库端口号/数据库名称
MAX_CONNECTIONS=10

# JWT配置
JWT_SECRET=EXAMPLE_JWT_SRCRET_KEY
JWT_EXPIRATION=24h

# 日志配置
RUST_LOG=info,questionnaire_backend=debug
```


## 许可

MIT 