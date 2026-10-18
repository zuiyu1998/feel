# feel — 文档目录

> 印象（feel）是减少人与人沟通与协作成本的平台：通过用户表达与客观评价，精准刻画用户画像（见 [主要体验](主要体验.md)）。
> 本页是按**项目架构**组织的文档总目录；每个 crate 的文档目录结构与其代码结构一一对应。仓库根 [README](../README.md) 提供快速入门。

## 1. 项目架构总览

Workspace 由 6 个 crate 组成（Cargo workspace，见根 `Cargo.toml`）：

```
导入层（可执行程序）
  cmd/feel_api ── Poem HTTP 服务：路由、Handler、DTO、JWT 认证
        │
        ▼
数据访问抽象层
  crates/feel_storage ── Repo trait（持久化接口）
                       └─ DataBase（业务数据访问网关，组合 Repo + 缓存）
  crates/feel_entity  ── 领域模型（UserBase / LabelBase / UserLabel 等，纯数据结构）
  crates/feel_core    ── 核心基础依赖（chrono / tokio 统一导出）
        │
        ▼
持久化实现层
  crates/feel_sea_orm ── Repo 的 Sea-ORM 实现 + ORM 实体（users / label / user_label …）
  crates/migration    ── Sea-ORM Migrator 数据库迁移脚本（启动时自动执行）

数据存储
  PostgreSQL（业务数据） + Redis（缓存）
```

依赖方向：`feel_api → feel_storage → feel_sea_orm → PostgreSQL`；`feel_entity`/`feel_core` 被各层共享；`migration` 由 `feel_api` 启动时调用（`Migrator::up`）。

| Crate | 职责 | 代码位置 | 文档位置 |
|-------|------|----------|----------|
| `feel_api` | HTTP API 服务（唯一的二进制入口） | `cmd/feel_api/` | [`feel_api/`](feel_api/) |
| `feel_entity` | 领域模型定义 | `crates/feel_entity/` | [`feel_entity/`](feel_entity/) |
| `feel_storage` | 持久化接口 + 数据访问网关 + 缓存 | `crates/feel_storage/` | [`feel_storage/`](feel_storage/) |
| `feel_sea_orm` | Sea-ORM 实体与 Repo 实现 | `crates/feel_sea_orm/` | [`feel_sea_orm/`](feel_sea_orm/) |
| `migration` | 数据库迁移 | `crates/migration/` | [`migration/`](migration/) |
| `feel_core` | 核心公共依赖 | `crates/feel_core/` | [`feel_core/`](feel_core/) |

## 2. 产品与特性文档

| 文档 | 内容 |
|------|------|
| [主要体验.md](主要体验.md) | 平台愿景、印象理念与功能规划（用户系统 / 标签系统 / 文章） |
| [features/label.md](features/label.md) | Label 用户标签特性的全链路设计：领域模型、业务规则、核心交互、未来扩展 |
| [features/user.md](features/user.md) | User 用户系统的整体设计：身份、凭据、认证、数据模型与接口对应 |
| [features/content.md](features/content.md) | Content 内容系统（文章 / 视频 …）设计方案：可扩展的类型建模、数据模型、API、扩展机制与实施计划 |

特性文档横跨整条链路（entity → storage → sea_orm → api），与各层模块文档互为补充：查"这个特性怎么设计"看 features 文档，查"这个类型/函数怎么实现"看对应层文档。

## 3. 分层文档目录

### 3.1 `feel_api` — HTTP 服务层（`cmd/feel_api/`）

| 文档 | 内容 |
|------|------|
| [feel_api/README.md](feel_api/README.md) | 服务概述、运行方式、ApiConfig / AppState、JWT 认证中间件、路由总表、整体架构 |
| [feel_api/deploy.md](feel_api/deploy.md) | 启动与部署：环境要求、环境变量、启动步骤、代码入口、依赖关系 |
| [feel_api/user.md](feel_api/user.md) | 用户接口：注册 / 注销 / 登录 / 登出 / 用户信息（逐接口：请求、响应、调用链） |
| [feel_api/label.md](feel_api/label.md) | 标签接口：create / all / list / users / add / update / remove（逐接口文档） |
| [feel_api/content.md](feel_api/content.md) | 内容接口：create / info / list / feed / update / remove（逐接口文档） |
| [feel_api/model/README.md](feel_api/model/README.md) | DTO 模块结构（`model/`）与设计原则 |
| [feel_api/model/user.md](feel_api/model/user.md) | 用户 DTO：各接口请求/响应模型定义 |
| [feel_api/model/label.md](feel_api/model/label.md) | 标签 DTO：各接口请求/响应模型定义（`LabelResponse`、`UserLabelItem` 等） |
| [feel_api/model/content.md](feel_api/model/content.md) | 内容 DTO：请求/响应模型（`ContentItem`、`ContentPageResponse` 等） |
| [feel_api/model/response.md](feel_api/model/response.md) | `ApiResponse<T>` 通用 JSON 响应封装、预定义状态码、工厂函数 |

### 3.2 `feel_entity` — 领域实体层（`crates/feel_entity/`）

| 文档 | 内容 |
|------|------|
| [feel_entity/README.md](feel_entity/README.md) | 领域实体总览：模块结构、`user` / `label` / `content` 模块、类型关系图、与 ORM 实体的关系 |

### 3.3 `feel_storage` — 数据访问层（`crates/feel_storage/`）

**用户模块**

| 文档 | 内容 |
|------|------|
| [feel_storage/UserRepo.md](feel_storage/UserRepo.md) | `UserRepo` trait：持久化层接口定义 |
| [feel_storage/SeaOrmUserRepo.md](feel_storage/SeaOrmUserRepo.md) | `UserRepo` 的 Sea-ORM 实现（register / unregister 详解） |
| [feel_storage/UserDataBase.md](feel_storage/UserDataBase.md) | `UserDataBase` trait：用户数据访问接口 |
| [feel_storage/CommonUserDataBase.md](feel_storage/CommonUserDataBase.md) | `UserDataBase` 默认实现（委托 Repo + 缓存） |
| [feel_storage/UserCache.md](feel_storage/UserCache.md) | `UserCache` 缓存接口与默认实现 |

**标签模块**

| 文档 | 内容 |
|------|------|
| [feel_storage/LabelRepo.md](feel_storage/LabelRepo.md) | `LabelRepo` trait：标签持久化层接口定义 |
| [feel_storage/SeaOrmLabelRepo.md](feel_storage/SeaOrmLabelRepo.md) | 标签持久化 Sea-ORM 实现（LabelBase / UserLabel 操作） |
| [feel_storage/LabelDataBase.md](feel_storage/LabelDataBase.md) | `LabelDataBase` trait：标签数据访问接口 |
| [feel_storage/CommonLabelDataBase.md](feel_storage/CommonLabelDataBase.md) | `LabelDataBase` 默认实现 |

**内容模块**

| 文档 | 内容 |
|------|------|
| [feel_storage/ContentRepo.md](feel_storage/ContentRepo.md) | `ContentRepo` trait：内容持久化接口（基表 + 类型明细，类型分派点） |
| [feel_storage/ContentDataBase.md](feel_storage/ContentDataBase.md) | `ContentDataBase` trait + `CommonContentDataBase`：业务校验与分页归一化 |

**错误与工具**

| 文档 | 内容 |
|------|------|
| [feel_storage/error.md](feel_storage/error.md) | `feel_storage::Error` 统一错误类型、变体来源，以及与 `from_storage_error` 的 HTTP 状态码映射 |
| [feel_storage/utils.md](feel_storage/utils.md) | `utils::get_passwod_hash`：Argon2 密码加盐哈希工具函数 |

### 3.4 `feel_sea_orm` — 持久化实现层（`crates/feel_sea_orm/`）

| 文档 | 内容 |
|------|------|
| [feel_sea_orm/UserEntity.md](feel_sea_orm/UserEntity.md) | `users` 表 ORM 实体：Model / Relation / ActiveModel / 类型转换 UserModel → UserBase |
| [feel_sea_orm/UserCredentialsEntity.md](feel_sea_orm/UserCredentialsEntity.md) | `user_credentials` 表 ORM 实体：登录凭据加密存储 |
| [feel_sea_orm/LabelEntity.md](feel_sea_orm/LabelEntity.md) | `label` 表 ORM 实体：Model / 类型转换 LabelModel → LabelBase |
| [feel_sea_orm/UserLabelEntity.md](feel_sea_orm/UserLabelEntity.md) | `user_label` 表 ORM 实体：用户×标签关联、唯一索引 |
| [feel_sea_orm/ContentEntity.md](feel_sea_orm/ContentEntity.md) | `content` 表 ORM 实体：内容基表与索引 |
| [feel_sea_orm/ContentArticleEntity.md](feel_sea_orm/ContentArticleEntity.md) | `content_article` 表 ORM 实体：文章明细 |
| [feel_sea_orm/ContentVideoEntity.md](feel_sea_orm/ContentVideoEntity.md) | `content_video` 表 ORM 实体：视频明细 |

### 3.5 `migration` — 数据库迁移（`crates/migration/`）

| 文档 | 内容 |
|------|------|
| [migration/README.md](migration/README.md) | 迁移总览：Migrator 定义、5 个迁移文件清单（含 content）、表结构概要、主键与自增约定、CLI 用法 |
| `crates/migration/README.md`（仓库内） | Migrator CLI 用法（generate / up / down / fresh / status 等命令） |

### 3.6 `feel_core` — 核心公共依赖（`crates/feel_core/`）

| 文档 | 内容 |
|------|------|
| [feel_core/README.md](feel_core/README.md) | 核心公共依赖：`chrono` / `tokio` 统一导出与使用方式 |

## 4. 文档现状与后续建议

### 已补齐（本次）

| 文档 | 状态 | 说明 |
|------|------|------|
| `docs/feel_sea_orm/LabelEntity.md` | ✅ | `label` 表实体（含 `From<Model> for LabelBase`） |
| `docs/feel_sea_orm/UserLabelEntity.md` | ✅ | `user_label` 关联表实体（联合唯一索引） |
| `docs/feel_sea_orm/UserCredentialsEntity.md` | ✅ | `user_credentials` 凭据表实体 |
| `docs/feel_storage/error.md` | ✅ | `feel_storage::Error` 统一错误与 API 状态码映射 |
| `docs/feel_storage/utils.md` | ✅ | `get_passwod_hash`（Argon2 密码哈希）工具函数 |
| `docs/migration/README.md` | ✅ | 迁移清单、表结构概要、主键约定、CLI |
| `docs/feel_core/README.md` | ✅ | 核心公共依赖说明 |
| `docs/feel_api/model/label.md` | ✅ | 标签 API 请求/响应 DTO 文档 |
| `docs/features/user.md` | ✅ | 用户系统特性设计文档 |
| `docs/features/content.md` | ✅ | 内容系统设计方案：基表+明细表建模、文章/视频、6 个接口、类型扩展机制与实施计划 |
| `docs/feel_api/content.md` | ✅ | 内容接口逐接口文档（create / info / list / feed / update / remove） |
| `docs/feel_api/model/content.md` | ✅ | 内容 DTO 请求/响应模型 |
| `docs/feel_storage/ContentRepo.md` | ✅ | 内容持久化接口与 SeaOrm 实现 |
| `docs/feel_storage/ContentDataBase.md` | ✅ | 内容数据访问接口与默认实现 |
| `docs/feel_sea_orm/ContentEntity.md` / `ContentArticleEntity.md` / `ContentVideoEntity.md` | ✅ | 内容基表与文章/视频明细表 ORM 实体 |
| 内容系统代码落地 | ✅ | Phase 1：migration + feel_entity/feel_sea_orm/feel_storage/feel_api 全链路，`cargo check --workspace` 通过 |
| 根 `README.md` | ✅ | 仓库入口：架构速览、快速开始、文档导航 |
| 既有文档刷新 | ✅ | `feel_api/README.md`（模块树/AppState）、`feel_api/model/README.md`、`UserEntity.md`（主键说明）、`features/label.md`（接口路径与 JSON）对齐当前实现 |

### 后续建议（可选）

| 优先级 | 建议 | 说明 |
|--------|------|------|
| 低 | API 参考集中页 | 将 `feel_api/` 各接口文档汇总为一页可检索的接口索引（含请求/响应示例） |
| 低 | features 文档扩展 | 评论（情绪化表达限制）、内容审核/推荐流等特性的设计文档，格式对齐 `features/user.md` / `features/label.md`（用户/标签/内容系统设计文档均已就绪，内容系统 Phase 1 代码已落地） |
| 低 | 部署细化 | 补充 Docker / CI 等部署方式（`deploy.md` 目前覆盖本地启动） |

## 5. 文档编写约定

1. **目录与代码结构对齐**：新模块文档放在 `docs/<crate>/` 下，文件名与 Rust 模块/类型名一致（如 `LabelRepo.md`、`UserEntity.md`）。
2. **结构统一**：概述 → 模块位置 → 定义（结构体/trait）→ 架构关系 → 使用示例 → 当前状态（✅ 已实现 / 🚧 规划中）。
3. **分层呼应**：特性级设计（`features/`）写"为什么"，模块级文档（`<crate>/`）写"是什么、怎么用"，两者互相链接。
4. **本目录同步维护**：新增/移动文档后，请更新本页对应表格。
