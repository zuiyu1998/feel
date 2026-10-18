# Content API — 内容接口

## 概述

内容系统（文章 / 视频 …）的 HTTP 接口，设计见 [features/content.md](../features/content.md)。实现位于：

- `cmd/feel_api/src/content/mod.rs` — handler 与路由
- `cmd/feel_api/src/model/content.rs` — 请求/响应 DTO

所有接口返回统一的 `ApiResponse<T>`（`code` / `message` / `data`），参数通过 `Json` 请求体携带（沿用项目约定，不使用 Path/Query）。

## 接口一览

| 方法 | 路径 | 鉴权 | Handler | 说明 |
|------|------|------|---------|------|
| POST | `/api/v1/content/create` | ✅ Bearer | `create_content` | 创建内容（文章/视频） |
| GET | `/api/v1/content/info` | ❌ | `content_info` | 查看内容详情 |
| GET | `/api/v1/content/list` | ❌ | `list_content` | 某用户的内容列表（分页） |
| GET | `/api/v1/content/feed` | ❌ | `content_feed` | 全站内容流（分页，可按类型过滤） |
| PUT | `/api/v1/content/update` | ✅ Bearer | `update_content` | 更新标题/载荷（仅作者） |
| DELETE | `/api/v1/content/remove` | ✅ Bearer | `remove_content` | 软删除（仅作者） |

## POST `/api/v1/content/create` — 创建内容

**请求（文章）：**

```json
{
    "content_type": "article",
    "title": "Rust 异步生态入门",
    "payload": { "body": "# 标题\nMarkdown 正文……" }
}
```

**请求（视频）：**

```json
{
    "content_type": "video",
    "title": "5 分钟看懂 SeaORM",
    "payload": {
        "video_url": "https://cdn.example.com/v/seaorm.mp4",
        "cover_url": "https://cdn.example.com/v/seaorm.webp",
        "duration_secs": 302
    }
}
```

**响应 `ApiResponse<ContentItem>`：**

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
        "created_at": "2026-02-10T00:00:00Z",
        "updated_at": "2026-02-10T00:00:00Z",
        "payload": { "body": "# 标题\nMarkdown 正文……" }
    }
}
```

## GET `/api/v1/content/info` — 查看内容详情

**请求：**

```json
{ "content_id": 1 }
```

**响应：** `ApiResponse<ContentItem>`（同上面的 data）。

## GET `/api/v1/content/list` — 某用户的内容列表

**请求：**

```json
{ "user_id": 42, "page": 1, "page_size": 20 }
```

**响应 `ApiResponse<ContentPageResponse>`：**

```json
{
    "code": 0,
    "message": "ok",
    "data": {
        "items": [ { "id": 1, "content_type": "article", "title": "…", "payload": { "body": "…" }, "…": "…" } ],
        "total": 57,
        "page": 1,
        "page_size": 20
    }
}
```

## GET `/api/v1/content/feed` — 全站内容流

**请求：**

```json
{ "content_type": "article", "page": 1, "page_size": 20 }
```

**响应：** `ApiResponse<ContentPageResponse>`（同 list）。

## PUT `/api/v1/content/update` — 更新内容（仅作者）

**请求：**

```json
{ "content_id": 1, "title": "新标题", "payload": { "body": "更新后的正文……" } }
```

**响应：** `ApiResponse<ContentItem>`（更新后的内容）。

## DELETE `/api/v1/content/remove` — 软删除（仅作者）

**请求：**

```json
{ "content_id": 1 }
```

**响应：** `ApiResponse<()>`（`data: null`）。

## 业务校验

标题与载荷在 `CommonContentDataBase` 校验（详见 [features/content.md](../features/content.md)「业务规则」），失败返回业务错误 `10006`：

| 校验项 | 规则 |
|--------|------|
| `title` | 非空白，去首尾空白后 1~100 字符 |
| `payload`（article） | `body` 非空白，≤ 100,000 字符 |
| `payload`（video） | `video_url` 必填且 `http(s)://` 开头；`cover_url` 可选，同前缀要求；`duration_secs` ≥ 0 |
| 归属校验 | update / remove 仅允许作者本人操作 |
| 可见性 | 软删除内容视为不存在（`10004`）；列表只返回 `enabled` 且未删除内容 |

## 错误映射

复用 `from_storage_error`（见 [model/response.md](model/response.md)）：业务校验失败 → `10006`，内容不存在 → `10004`，数据库异常 → `10003`。

## 路由挂载与 AppState

```rust
// cmd/feel_api/src/lib.rs
Route::new()
    .nest("/users", user::router())
    .nest("/labels", label::router())
    .nest("/content", content::router())
```

`AppState` 提供 `content_database: Arc<dyn ContentDataBase>`（`CommonContentDataBase`，内含 `SeaOrmContentRepo`）。

## 涉及的 crate 依赖

| crate / 模块 | 作用 |
|--------------|------|
| `feel_api::model::content` | 请求/响应 DTO（`CreateContentRequest`、`ContentItem`、`ContentPageResponse` 等） |
| `feel_entity::content` | 领域模型（`ContentBase`、`ContentKind`、载荷、分页） |
| `feel_storage` | `ContentDataBase` / `ContentRepo` 及其实现 |
| `feel_sea_orm::content` | `content` / `content_article` / `content_video` ORM 实体 |
| `migration` | `m20260210_000000_add_content` 建表迁移 |
