# ContentRepo — 内容持久化层接口

## 概述

`ContentRepo` 是 `feel_storage` 中定义的**内容持久化接口**（trait 位于 `crates/feel_storage/src/repo/mod.rs`），由 `crates/feel_storage/src/repo/content.rs` 的 `SeaOrmContentRepo` 实现。

内容系统的基表 + 类型明细读写都在此层完成：**`ContentKind` 类型分派集中于此**（创建时写明细、读取时组装类型载荷）。这是新增内容类型时的主要修改点之一。

## 模块位置

```
crates/feel_storage/src/repo/
├── mod.rs        # UserRepo / LabelRepo / ContentRepo trait 定义
├── user.rs       # SeaOrmUserRepo
├── label.rs      # SeaOrmLabelRepo
└── content.rs    # SeaOrmContentRepo ←
```

## Trait 定义

```rust
#[async_trait]
pub trait ContentRepo: 'static + Send + Sync {
    async fn create(&self, create: &ContentCreate) -> Result<Content>;
    async fn find_by_id(&self, content_id: i64) -> Result<Option<Content>>;
    async fn find_by_user(&self, user_id: i64, offset: u64, limit: u64) -> Result<Vec<Content>>;
    async fn count_by_user(&self, user_id: i64) -> Result<u64>;
    async fn find_feed(&self, content_type: Option<&str>, sort: FeedSort, offset: u64, limit: u64) -> Result<Vec<Content>>;
    async fn count_feed(&self, content_type: Option<&str>) -> Result<u64>;
    async fn increment_view_count(&self, content_id: i64) -> Result<u64>;
    async fn update(&self, update: &ContentUpdate) -> Result<Content>;
    async fn soft_delete(&self, content_id: i64, user_id: i64) -> Result<()>;
}
```

| 方法 | 说明 |
|------|------|
| `create` | 事务内写 `content` 基表 + 按类型写明细表；返回组装后的 `Content` |
| `find_by_id` | 查基表 + 按 `content_type` 查对应明细（1:1），组装 `Content`；明细缺失返回 None |
| `find_by_user` | 某用户 public 内容（`enabled` 且未删除），按创建时间倒序，offset/limit 分页 |
| `count_by_user` | 对应列表的总数 |
| `find_feed` | 全站 Feed 流，可按 `content_type` 过滤、按 `FeedSort`（latest/popular）排序 |
| `count_feed` | 内容流总数 |
| `increment_view_count` | 浏览量原子 +1（feed 热度数据源；不存在/已删除返回 0） |
| `update` | 更新标题 / 载荷（明细行 upsert），返回更新后的 `Content` |
| `soft_delete` | 置 `is_delete = true`（带 `user_id` 归属过滤） |

## 默认实现：SeaOrmContentRepo

- `new(conn)` 注入共享 `DatabaseConnection`
- **创建**：`ContentKind::parse(content_type, payload)` 解析载荷 → 事务中插入基表取回自增 `id` → 按类型插入 `content_article` / `content_video`
- **批量读取**：先查基表分页，再按类型收集 `content_id` 列表，各明细表一次 `IN` 查询装入 `HashMap`，避免 N+1；明细缺失的脏数据行跳过
- **更新**：明细存在则 update，不存在则 insert（upsert）
- **软删除**：先按主键取行并校验 `user_id`，再置 `is_delete = true`
- **Feed 排序**：`FeedSort::Latest` 按 `created_at` 倒序；`FeedSort::Popular` 按 `view_count` 倒序（同浏览量按创建时间倒序）
- **浏览量**：`increment_view_count` 以 `UPDATE ... SET view_count = view_count + 1` 原子递增（同时过滤 `is_delete`）

## 架构关系

```
ContentDataBase（业务层：校验 + 分页）
    ↓ 委托
ContentRepo（持久化接口）
    ↓ 实现
SeaOrmContentRepo ── sea-orm ──► content / content_article / content_video
```

## 使用示例

```rust
use feel_storage::repo::{ContentRepo, SeaOrmContentRepo};

let repo = SeaOrmContentRepo::new(conn);
let content = repo.create(&create).await?;
```

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| `create` | ✅ | 事务 + 类型分派 |
| `find_by_id` | ✅ | 基表 + 明细组装 |
| `find_by_user` / `count_by_user` | ✅ | 分页（offset/limit） |
| `find_feed` / `count_feed` | ✅ | 可按类型过滤、最新/热门排序 |
| `increment_view_count` | ✅ | 浏览量原子递增 |
| `update` | ✅ | 标题/载荷 upsert |
| `soft_delete` | ✅ | 归属过滤 + `is_delete` |
