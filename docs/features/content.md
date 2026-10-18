# Content — 内容系统（文章 / 视频 …）

## 概述

内容系统承载用户的"理智表达"——文章、视频等长内容，是平台的核心产出物。印象的定位是"通过表达描绘用户画像"，内容系统正是这一表达的主渠道（与标签的"轻量表达"互补）。系统还需为后续的评论、点赞、收藏、话题关联等提供内容锚点（`content.id`）。

设计目标中最重要的约束：**内容类型可扩展**——当前实现文章（article）与视频（video），未来可平滑加入图集、播客、投票、外链分享等类型，而**不改动核心表结构与 API 框架**。

## 设计目标

1. **类型可扩展** — 新增内容类型不触碰基表与路由框架，只需"1 个明细表 + 1 个领域枚举变体 + 1 处类型分派"
2. **公共字段与类型字段分离** — 公共字段进 `content` 基表，类型专属字段进各自明细表（class-table inheritance）
3. **归属与权限** — 内容归属作者（`user_id` 关联 `users.id`），写操作仅作者本人可执行，认证复用现有 JWT 中间件
4. **软删除** — `is_delete` 标记删除，历史可追溯（与 `users` 表的先例一致）
5. **统一出口** — 所有类型对外统一返回 `ContentItem`（基表字段 + `payload` 对象），客户端按 `content_type` 解析
6. **可分页的 Feed 流** — 支撑全站 feed（最新/热门排序）与用户主页列表，列表项内嵌作者摘要

## 建模选型

| 方案 | 做法 | 优点 | 缺点 | 结论 |
|------|------|------|------|------|
| **A. 基表 + 明细表**（class-table inheritance） | `content` 存公共字段；`content_article` / `content_video` 存类型字段，1:1 关联 `content.id` | 类型字段强类型、可建列级索引与约束；扩展=新增明细表；与 SeaORM 实体风格一致 | 表数量随类型增长；读取需附加一次明细查询 | ✅ **采用** |
| B. 单表 + `payload JSONB` | `content` 一表，类型字段塞入 JSONB | 表最少、扩展最快（加个 serde 模型即可） | 类型弱、难建列索引、校验只能在应用层、跨类型统计困难 | 备选（类型数量爆发式增长时再评估） |
| C. 每类型一张独立表 | `article` / `video` 各自独立建表 | 单表查询最直接 | 跨类型列表需 UNION；公共逻辑（权限/软删/分页）重复 | 不采用 |

> 本项目是 SeaORM + PostgreSQL 强类型风格（现有 `users` / `user_label` 均一表一实体），方案 A 与该架构最契合，同时保留清晰的扩展面。

## 领域模型（feel_entity::content）

```rust
pub const CONTENT_TYPE_ARTICLE: &str = "article";
pub const CONTENT_TYPE_VIDEO: &str = "video";

/// 内容公共部分（基表）
pub struct ContentBase {
    pub id: i64,
    pub user_id: i64,          // 作者（引用 UserBase.id）
    pub content_type: String,  // "article" | "video" | 未来扩展
    pub title: String,
    pub enabled: bool,         // 可见性/管理开关
    pub is_delete: bool,       // 软删除标记
    pub view_count: i64,       // 浏览量（Feed 热度排序依据）
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 文章明细
pub struct ArticlePayload {
    pub body: String,          // Markdown 正文
}

/// 视频明细
pub struct VideoPayload {
    pub video_url: String,     // 视频地址（对象存储/CDN）
    pub cover_url: String,     // 封面图（可空串）
    pub duration_secs: i64,    // 时长（秒），未知时为 0
}

/// 类型化载荷（扩展点）
pub enum ContentKind {
    Article(ArticlePayload),
    Video(VideoPayload),
}

/// 组合视图：基表 + 类型载荷
pub struct Content {
    pub base: ContentBase,
    pub kind: ContentKind,
}

/// 创建/更新请求（payload 承载类型专属字段）
pub struct ContentCreate {
    pub user_id: i64,
    pub content_type: String,
    pub title: String,
    pub payload: serde_json::Value,   // HTTP 边界做类型校验后落库
}

pub struct ContentUpdate {
    pub id: i64,
    pub user_id: i64,                 // 由 handler 校验归属
    pub title: Option<String>,
    pub payload: Option<serde_json::Value>,
}

/// Feed 流排序方式
pub enum FeedSort {
    Latest,   // 最新（默认，按 createdAt 倒序）
    Popular,  // 热门（按 viewCount 倒序，同浏览量按 createdAt 倒序）
}
```

- `payload` 用 `serde_json::Value` 承载类型专属字段：API 层按 `content_type` 反序列化为具体 `ArticlePayload` / `VideoPayload` 校验后再写入明细表
- `ContentKind` 是扩展的**中心分派点**：仓储层通过 `match kind` 读写对应明细表（见「扩展机制」）

## 数据模型

### 表 `content`（基表，公共字段）

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGINT` | PK 自增 | 内容主键（各系统锚点） |
| `user_id` | `BIGINT` | NOT NULL, 索引 | 作者（关联 `users.id`） |
| `content_type` | `VARCHAR` | NOT NULL, 索引 | 内容类型（将来可加 CHECK） |
| `title` | `VARCHAR` | NOT NULL | 标题 |
| `enabled` | `BOOLEAN` | NOT NULL DEFAULT true | 是否可见/启用 |
| `is_delete` | `BOOLEAN` | NOT NULL DEFAULT false | 软删除标记 |
| `view_count` | `BIGINT` | NOT NULL DEFAULT 0 | 浏览量（Feed 热度排序依据） |
| `created_at` | `TIMESTAMPTZ` | NOT NULL | 创建时间 |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL | 更新时间 |

索引：

```sql
CREATE INDEX idx_content_user_created   ON content (user_id, created_at);
CREATE INDEX idx_content_type_created   ON content (content_type, created_at);
-- 浏览量（feed 热度），由 m20260211_000000_add_content_view_count 追加
CREATE INDEX idx_content_view_created   ON content (view_count, created_at);
```

### 表 `content_article`（文章明细，1:1）

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `content_id` | `BIGINT` | PK | 关联 `content.id` |
| `body` | `TEXT` | NOT NULL | Markdown 正文 |

### 表 `content_video`（视频明细，1:1）

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `content_id` | `BIGINT` | PK | 关联 `content.id` |
| `video_url` | `VARCHAR` | NOT NULL | 视频地址 |
| `cover_url` | `VARCHAR` | NOT NULL DEFAULT '' | 封面图地址 |
| `duration_secs` | `INTEGER` | NOT NULL DEFAULT 0 | 时长（秒） |

> 沿用项目约定：外键为**逻辑关联**（迁移中不建 DB 级 FK，与 `user_label` 一致）；`is_delete` 沿用 `users` 先例，`enabled` 沿用 `label` 先例。

## 业务规则

1. **类型白名单** — `content_type` 必须是已注册类型（当前 `article` / `video`），否则业务错误 `10006`
2. **归属校验** — 更新/删除时校验 `auth.uid → UserBase.id == content.user_id`，否则 `10006`
3. **软删除** — 删除仅置 `is_delete = true`；列表/信息默认过滤
4. **可见性** — `enabled = false` 的内容不在 feed / 用户列表公开返回（作者与管理端后续可配置可见范围）
5. **内容校验**

   | 字段 | 规则 |
   |------|------|
   | `title` | 非空白，1~100 字符 |
   | `body`（article） | 非空白，≤ 100,000 字符 |
   | `video_url` | 必填，`http(s)://` 开头（后续接入对象存储直传后改为预签名流程） |
   | `cover_url` | 可选，`http(s)://` 开头 |
   | `duration_secs` | ≥ 0 |

6. **分页** — `page` 从 1 开始；`page_size` 默认 20、上限 50；列表按 `created_at` 倒序（热门流按 `view_count` 倒序）

## API 设计

路由挂载于 `/api/v1` 下，沿用现有约定：参数统一走 `Json` 请求体（不使用 Path/Query）。

| 方法 | 路径 | 鉴权 | 说明 |
|------|------|------|------|
| POST | `/api/v1/content/create` | ✅ Bearer | 创建内容（文章/视频） |
| GET  | `/api/v1/content/info` | ❌ | 查看内容详情 |
| GET  | `/api/v1/content/list` | ❌ | 某用户的内容列表（分页） |
| GET  | `/api/v1/content/feed` | ❌ | 全站 Feed 流（分页，可按类型过滤，`sort=latest/popular`） |
| PUT  | `/api/v1/content/update` | ✅ Bearer | 更新标题/正文（仅作者） |
| DELETE | `/api/v1/content/remove` | ✅ Bearer | 软删除（仅作者） |

### 创建内容 — `POST /api/v1/content/create`

```json
{
    "content_type": "article",
    "title": "Rust 异步生态入门",
    "payload": { "body": "# 标题\nMarkdown 正文……" }
}
```

```json
{
    "content_type": "video",
    "title": "5 分钟看懂 SeaORM",
    "payload": { "video_url": "https://cdn.example.com/v/seaorm.mp4", "cover_url": "https://cdn.example.com/v/seaorm.webp", "duration_secs": 302 }
}
```

响应（`ApiResponse<ContentItem>`）：

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "id": 1,
        "user_id": 42,
        "content_type": "article",
        "title": "Rust 异步生态入门",
        "enabled": true,
        "view_count": 0,
        "created_at": "2026-02-10T00:00:00Z",
        "updated_at": "2026-02-10T00:00:00Z",
        "payload": { "body": "# 标题\nMarkdown 正文……" },
        "author": { "user_id": 42, "name": "张三", "avatar": "https://example.com/avatar.png" }
    }
}
```

### 列表 / 内容流

```json
// GET /api/v1/content/list
{ "user_id": 42, "page": 1, "page_size": 20 }

// GET /api/v1/content/feed（sort: "latest" 最新 | "popular" 热门，默认 latest）
{ "content_type": "article", "sort": "popular", "page": 1, "page_size": 20 }
```

响应（`ApiResponse<ContentPageResponse>`）：

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "items": [ { "id": 1, "content_type": "article", "title": "…", "view_count": 12, "author": { "user_id": 42, "name": "张三", "avatar": "…" }, "payload": { "body": "…" }, "…": "…" } ],
        "total": 57,
        "page": 1,
        "page_size": 20
    }
}
```

## Feed 流

Feed 流是全站内容的聚合时间线（用户主页列表 `list` 使用同一卡片结构）。

- **排序**：`sort=latest`（默认，按 `created_at` 倒序）/ `sort=popular`（按 `view_count` 倒序，同浏览量按 `created_at` 倒序）
- **卡片**：列表项内嵌 `author`（`user_id/name/avatar`）与 `view_count`，客户端无需二次查询
- **浏览量统计**：`GET /content/info` 成功返回时对内容执行一次 `view_count + 1`（best-effort，失败不影响详情返回）——热门排序的数据来源
- **分页**：`page/page_size`（默认 20，上限 50）；feed 滚动按 `page` 递增请求（未来可升级为游标分页）
- **过滤**：`content_type` 可选（article / video）
- **缓存**：feed 分页经 Redis 缓存（Cache-Aside，TTL 60s）；内容写操作（create/update/remove）以版本号（`INCR`）失效全部旧缓存；浏览量变化由 TTL 自然收敛（见 `docs/feel_storage/ContentCache.md`）

## 扩展机制：新增内容类型（示例：加入"播客"）

扩展面被刻意收敛为**固定清单**，按清单操作即可；由于 Rust 中 `match ContentKind` 必须穷尽，**漏改任意分派点会直接编译失败**，形成编译期保障：

1. **迁移** — 新增 `content_podcast` 明细表（`content_id` PK + `audio_url` / `duration_secs` 等）
2. **feel_entity** — 新增 `PodcastPayload` 结构体，并在 `ContentKind` 中加入 `Podcast(PodcastPayload)` 变体
3. **feel_sea_orm** — 新增 `ContentPodcast` 实体（`#[sea_orm(table_name = "content_podcast")]`），实现 `From<Model> for PodcastPayload`
4. **feel_storage/repo/content.rs** — 明细读写分派函数中各加一个 `match` 分支（`load_detail` / `save_detail` 各一处）
5. **feel_api** — 通常**无需改动**：`payload` 为 JSON 透传；若要类型级校验，在创建/更新处加一个 payload 反序列化分支
6. **文档** — 更新本页「类型清单」与 API 示例

> 未来若类型数量显著增多（>5~6 种），可把上述 `match` 抽象为注册表（`HashMap<&'static str, Box<dyn ContentTypeHandler>>`，注册 `save/load/validate`），启动时注入 `CommonContentDataBase`；这样新增类型只需新增一个实现模块并注册，不再修改分派函数。当前阶段不引入该抽象，避免过度设计。

## 分层落点（与现有架构的对应）

| 层 | 落点 | 说明 |
|----|------|------|
| `migration` | `m2026xxxx_add_content.rs` | 建 `content` / `content_article` / `content_video` |
| `feel_entity::content` | `crates/feel_entity/src/content/` | `ContentBase`、`ContentKind`、payload、Create/Update |
| `feel_sea_orm::content` | `crates/feel_sea_orm/src/content/entities/` | 3 个实体 + 转换 |
| `feel_storage::repo::ContentRepo` | `crates/feel_storage/src/repo/content.rs` | trait + `SeaOrmContentRepo`（分派点） |
| `feel_storage::database::ContentDataBase` | `crates/feel_storage/src/database/content.rs` | trait + `CommonContentDataBase`（校验 + 委托） |
| `feel_api::model::content` | `cmd/feel_api/src/model/content.rs` | 请求/响应 DTO |
| `feel_api::content` | `cmd/feel_api/src/content/mod.rs` | handler + 路由 |
| `AppState` | `cmd/feel_api/src/lib.rs` | 注入 `content_database: Arc<dyn ContentDataBase>` |

## 实施计划

**Phase 1 — MVP（文章 + 视频，可上线）** ✅ 已实现

1. ✅ 迁移：`content` / `content_article` / `content_video`
2. ✅ `feel_entity` 领域模型与 `ContentKind`
3. ✅ `feel_sea_orm` 实体与转换
4. ✅ `feel_storage`：`ContentRepo` / `SeaOrmContentRepo`、`ContentDataBase` / `CommonContentDataBase`（含类型校验与 payload 落库）
5. ✅ `feel_api`：DTO + 6 个接口 + `AppState` 注入 + 路由挂载
6. ✅ 文档：本页 + `docs/feel_api/content.md` + 各层模块文档 + 总目录更新（`cargo check --workspace` 通过）

> **Phase 1.5 — Feed 流增强** ✅ 已实现：`view_count` 统计（迁移 `m20260211_000000_add_content_view_count`）、`latest` / `popular` 排序、列表卡片内嵌作者摘要（`author`）、Redis 分页缓存（`ContentCache`：Cache-Aside + 版本号失效 + 60s TTL）。

**Phase 2 — 增强**

- 草稿/发布状态（`status` 列或独立草稿表）、可见范围（公开/粉丝/私密）
- 内容审核（管理员 `enabled` 开关 + 审核队列）
- 对象存储直传（预签名上传 URL）+ 视频转码异步任务
- 搜索（`title` 的 tsvector 索引；正文全文检索）
- feed 缓存 ✅ 已实现（Cache-Aside + 版本号失效 + 60s TTL，见「Feed 流」与 `docs/feel_storage/ContentCache.md`）

**Phase 3 — 生态**

- 评论 / 点赞 / 收藏 / 话题关联：以 `content_id` 为锚点，模式与 `user_label` 一致
- 用户主页聚合（内容 + 标签 + 简介）

## 与现有实现的关系

| 层 | 现状 | 本系统所需 |
|----|------|-----------|
| `feel_entity` | 已有 `UserBase` / `LabelBase` 等 | 新增 `content` 模块 |
| `feel_sea_orm` | 已有 users / label / user_label 实体 | 新增 3 个内容实体 |
| `feel_storage` | 已有 User/Label 的 Repo + DataBase 模式 | 沿用同一模式：`ContentRepo` + `ContentDataBase` |
| `feel_api` | 已有用户/标签接口与 JWT 认证 | ✅ 已新增 content 路由（复用 `auth_middleware`） |
| `migration` | 已有 4 个迁移 | ✅ 已新增 1 个迁移（3 张表） |
| 现有 API 规范 | JSON 请求体统一传参、`ApiResponse` 包装、错误码映射 | 完全复用 |

## 未来扩展

- 草稿箱与定时发布
- 内容版本历史（`content_revision` 明细表，文章编辑留痕）
- 内容类型：图集 / 播客 / 投票 / 外链分享
- 推荐流（基于标签 + 互动的排序）
- 内容举报与审核工作流
