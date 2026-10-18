# ContentDataBase — 内容数据访问接口

## 概述

`ContentDataBase` 是面向业务层的**内容数据访问接口**（位于 `crates/feel_storage/src/database/content.rs`），封装 `ContentRepo`，负责业务规则校验与分页归一化；默认实现 `CommonContentDataBase`。

## 模块位置

```
crates/feel_storage/src/database/
├── mod.rs        # 模块声明（re-export）
├── content.rs    # ContentDataBase trait + CommonContentDataBase ←
├── label.rs      # LabelDataBase trait + CommonLabelDataBase
└── user.rs       # UserDataBase trait + CommonUserDataBase
```

## Trait 定义

```rust
#[async_trait]
pub trait ContentDataBase: 'static + Send + Sync {
    async fn create(&self, create: &ContentCreate) -> Result<Content>;
    async fn get(&self, content_id: i64) -> Result<Option<Content>>;
    async fn list_by_user(&self, user_id: i64, page: Option<u64>, page_size: Option<u64>) -> Result<ContentPage>;
    async fn list_feed(&self, content_type: Option<&str>, page: Option<u64>, page_size: Option<u64>) -> Result<ContentPage>;
    async fn update(&self, update: &ContentUpdate) -> Result<Content>;
    async fn remove(&self, content_id: i64, user_id: i64) -> Result<()>;
}
```

| 方法 | 说明 |
|------|------|
| `create` | 校验标题/载荷 → 委托 `ContentRepo::create` |
| `get` | 查内容；已软删除的返回 `None`（对 API 表现为 404） |
| `list_by_user` / `list_feed` | 分页归一化（默认 20 条/页，最大 50）后委托 Repo；feed 支持 `sort`（latest/popular） |
| `update` | 归属校验（仅作者）→ 标题/载荷校验 → 委托 Repo |
| `remove` | 归属校验 → 委托 Repo 软删除 |
| `record_view` | 浏览量 +1（委托 `increment_view_count`，best-effort） |

## 业务校验规则

| 校验项 | 规则 | 错误 |
|--------|------|------|
| `title` | 去首尾空白后非空，≤ 100 字符（字符数） | `10006` |
| article `body` | 非空白，≤ 100,000 字符 | `10006` |
| video `video_url` | 必填，`http(s)://` 开头 | `10006` |
| video `cover_url` | 可空；非空时 `http(s)://` 开头 | `10006` |
| video `duration_secs` | ≥ 0 | `10006` |
| 归属 | update / remove 仅作者本人 | `10006` |
| 载荷解析失败 | 字段缺失/类型不符 | `10006` |

> 校验常量：`DEFAULT_PAGE_SIZE = 20`、`MAX_PAGE_SIZE = 50`、`MAX_TITLE_CHARS = 100`、`MAX_ARTICLE_BODY_CHARS = 100_000`（定义于 `database/content.rs`）。

## 默认实现：CommonContentDataBase

```rust
pub struct CommonContentDataBase {
    content_repo: Box<dyn ContentRepo>,
    content_cache: Box<dyn ContentCache>,
}

impl CommonContentDataBase {
    pub fn new<T: ContentRepo, C: ContentCache>(content_repo: T, content_cache: C) -> Self { /* ... */ }
}
```

- 构造时注入任意 `ContentRepo`（当前 `SeaOrmContentRepo`）与 `ContentCache`（当前 `CommonContentCache`）实现
- `paginate()`：把可选 `page/page_size` 归一化为 `(offset, page, page_size)`
- `list_feed()`：Cache-Aside —— 先查 `ContentCache`，未命中回源并把结果写回缓存（TTL 60s；缓存异常时降级回源）
- create / update / remove 成功后调用 `invalidate_feed()`（INCR 版本号，O(1) 失效全部旧缓存）
- 业务异常统一以 `crate::Error::Business(...)` 返回，API 层映射为状态码 `10006`

## 架构关系

```
Handler（feel_api::content）
    ↓
ContentDataBase（业务校验 + 分页）
    ↓ 缓存命中直接返回 / 未命中回源并回填
CommonContentDataBase ──► ContentCache（CommonContentCache → Redis）
    ↓ 委托
ContentRepo（SeaOrmContentRepo）→ content / content_article / content_video
```

## 使用示例

```rust
use feel_storage::cache::content::CommonContentCache;
use feel_storage::database::{CommonContentDataBase, ContentDataBase};

let cache = CommonContentCache::new(redis_client);
let db = CommonContentDataBase::new(SeaOrmContentRepo::new(conn), cache);
let page = db.list_by_user(42, Some(1), Some(20)).await?;
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 标题/载荷校验 | ✅ | `validate_title` / `validate_payload` |
| 归属校验 | ✅ | update / remove 仅作者 |
| 分页归一化 | ✅ | 默认 20 / 上限 50 |
| 与 ContentRepo 委托 | ✅ | 全部委托 |
| Feed 排序 / 浏览量 | ✅ | `FeedSort` 透传 + `record_view` |
| Feed 缓存 | ✅ | Cache-Aside（`ContentCache`）+ 版本号失效 + 60s TTL |
