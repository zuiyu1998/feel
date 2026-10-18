# User — 用户系统

## 概述

用户是印象平台的主体：文章、评论、标签最终都归属于用户。本文档描述用户系统的整体设计——身份、凭据、认证与数据模型；接口细节见 [feel_api/user.md](../feel_api/user.md)，各层实现见对应模块文档。

## 设计目标

1. **唯一身份** — `users.uid` 为全局唯一业务键（`typedflake::id!(UserId)` 生成），主键 `id` 由数据库序列自增，作为跨表关联（如 `user_label.user_id`、`user_credentials.user_uid` 的逻辑锚点）的稳定内部标识
2. **软删除** — 注销（unregister）仅置 `is_delete = true`，历史数据（文章、标签关联）不因用户注销而丢失
3. **凭据分离** — 用户主体（`users`）与登录凭据（`user_credentials`）分表，支持一个用户绑定多种凭据（密码 / 邮箱 / 手机）
4. **安全存储** — 密码经 Argon2 加盐哈希后写入 `user_credentials.encrypted_data`，盐写入 `encryption_key`
5. **无状态认证** — 登录签发 JWT Bearer token，写操作经 `auth_middleware` 校验

## 领域模型（feel_entity::user）

| 类型 | 说明 |
|------|------|
| `UserBase` | 用户主体：`id` / `uid` / `name` / `avatar` / `slogan` / `enabled` / `created_at` / `updated_at` |
| `UserRegister` | 注册请求合并体：`name` / `avatar` / `credential_type` / `credential_name` / `data`（明文） |
| `UserLogin` | 登录凭据：`credential_name` / `data` |
| `LoginResult` | 登录结果：`token` + `user_base` |
| `UserCredential` | 凭据实体：`id` / `user_uid` / `credential_type` / `credential_name` / `encrypted_data` / `encryption_key` / `enabled` / 时间戳 |

## 数据模型

### 表 `users`

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGINT` | PK 自增 | 内部主键 |
| `uid` | `VARCHAR` | UNIQUE | 全局唯一业务键（typedflake） |
| `name` | `VARCHAR` | NOT NULL | 用户昵称 |
| `avatar` | `VARCHAR` | NOT NULL | 头像 URL |
| `slogan` | `VARCHAR` | 默认空 | 签名/简介 |
| `is_enable` | `BOOLEAN` | 默认 true | 是否启用 |
| `is_delete` | `BOOLEAN` | 默认 false | 软删除标记 |
| `created_at` / `updated_at` | `TIMESTAMPTZ` | NOT NULL | 时间戳 |

### 表 `user_credentials`

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGINT` | PK 自增 | 主键 |
| `user_uid` | `VARCHAR` | 索引 | 用户业务键，逻辑关联 `users.uid` |
| `credential_type` | `VARCHAR` | 索引 | 凭据类型（password / email / phone） |
| `credential_name` | `VARCHAR` | | 凭据名/标识 |
| `encrypted_data` | `VARCHAR` | | Argon2 哈希后的凭据数据 |
| `encryption_key` | `VARCHAR` | | 盐（加密密钥标识） |
| `is_enabled` | `BOOLEAN` | 默认 true | 是否启用 |
| `created_at` / `updated_at` | `TIMESTAMPTZ` | NOT NULL | 时间戳 |

> 唯一约束：`(user_uid, credential_name)` 联合唯一——同一用户同一凭据名只能有一条。

## 业务规则

1. **注册即建主体 + 凭据** — 注册在同一事务内插入 `users` 行与 `user_credentials` 行，任一失败整体回滚
2. **uid 与 id 分工** — 对外/跨表使用 `uid`（业务键）；内部关联使用 `id`（主键）
3. **登录校验** — 按 `credential_name` 查凭据 → Argon2 `verify_password` 校验 → 查 `users` 行（成功后签发 token 并写缓存）
4. **注销为软删除** — `unregister` 仅置 `is_delete = true`，不物理删除；是否在查询侧过滤软删除由业务查询条件决定（当前查询不额外过滤）
5. **写操作认证** — `auth_middleware` 从 `Authorization: Bearer <token>` 解析 uid 注入 `AuthUser`

## 核心交互（实际接口）

### 注册 — `POST /api/v1/users/register`

```json
{
    "name": "张三",
    "avatar": "https://example.com/avatar.png",
    "credential_type": "password",
    "credential_name": "default",
    "data": "my_secret_password"
}
```

返回用户主体信息（`id` / `uid` / `name` / `avatar` / `slogan` / `enabled` / 时间戳）。

### 注销 — `POST /api/v1/users/unregister/:user_id`

按主键注销（软删除）。

### 登录 — `POST /api/v1/users/login`

```json
{ "credential_name": "alice@example.com", "data": "my_password" }
```

成功返回 `{ "token": "…" }`；凭据错误返回 `10005`（Authentication error）。

### 用户信息 — `GET /api/v1/users/info/:uid`

按 uid 查询公开信息（未登录可访问）。

### 登出 — `POST /api/v1/users/logout`

当前为占位实现（无状态 JWT 的登出通常由客户端丢弃 token 或服务端加入黑名单）。

## 与现有实现的关系

| 层 | 现状 |
|----|------|
| `feel_entity::user` | ✅ `UserBase` / `UserRegister` / `UserLogin` / `LoginResult` / `UserCredential` |
| `feel_sea_orm` | ✅ `users`、`user_credentials` ORM 实体（`UserModel → UserBase`） |
| `feel_storage` | ✅ `UserRepo` / `SeaOrmUserRepo`（register / unregister / login / 查询）、`UserDataBase` / `CommonUserDataBase`（缓存 + JWT）、`UserCache`、`utils::get_passwod_hash`（Argon2） |
| `feel_api` | ✅ 注册 / 注销 / 登录 / 用户信息接口，`auth_middleware` JWT 认证 |
| `migration` | ✅ `users`、`user_credentials` 建表迁移 |

各层细节见 [feel_api/user.md](../feel_api/user.md)、[feel_api/model/user.md](../feel_api/model/user.md)、[feel_storage/UserRepo.md](../feel_storage/UserRepo.md)、[feel_storage/error.md](../feel_storage/error.md) 等。

## 设计说明

1. **凭据多态** — `credential_type` 区分密码 / 邮箱 / 手机等，扩展登录方式无需改表结构
2. **展示与身份分离** — `name` / `avatar` / `slogan` 是可修改的展示信息；`uid` / `id` 是稳定身份
3. **注销不删数据** — 软删除保留"人设"与历史数据，符合"客观评价精准描述个人画像"的产品目标
4. **token 只含 uid** — JWT payload 仅存 uid（`sub`），减少泄漏面；用户数据以缓存/数据库为准

## 未来扩展

- 修改资料（`UserUpdate` 占位）、头像上传
- 多凭证绑定 / 登录方式切换
- 凭据轮换、登录风控、token 吊销黑名单
