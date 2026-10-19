# Follow — 用户关注系统

## 概述

关注系统建立用户间的单向关系（A 关注 B），是"认识用户"与内容社交分发的基础设施，为后续关注流、粉丝通知、主页聚合提供数据支撑。与 `user_label`（用户×标签多对多）同构：关联表 `user_follow` + 幂等的关注/取关操作。

## 设计目标

1. **单向关注** — `user_follow { user_id (关注者), follow_user_id (被关注者) }`，无互相关注概念（双向关注 = 两条记录）
2. **幂等** — 重复关注返回既有关系；重复取关静默成功
3. **防自关注** — 不能关注自己（业务错误 `10006`）
4. **分页列表** — 粉丝 / 关注列表分页（默认 20，上限 50），按创建时间倒序
5. **统计** — 粉丝数 / 关注数；可选认证时附带 `is_following`
6. **公开可见** — 当前关注/粉丝列表公开（隐私设置列为未来扩展）

## 领域模型（feel_entity::follow）

```rust
pub struct UserFollow {
    pub id: i64,
    pub user_id: i64,          // 关注者（引用 UserBase.id）
    pub follow_user_id: i64,   // 被关注者（引用 UserBase.id）
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct UserFollowCreate { pub user_id: i64, pub follow_user_id: i64 }

pub struct FollowStats {
    pub user_id: i64,
    pub follower_count: u64,        // 粉丝数
    pub following_count: u64,       // 关注数
    pub is_following: Option<bool>, // 请求者是否关注该用户（匿名/本人为 None）
}

pub struct FollowPage {
    pub items: Vec<UserFollow>,
    pub total: u64,
    pub page: u64,
    pub page_size: u64,
}
```

## 数据模型

### 表 `user_follow`

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGINT` | PK 自增 | 关联主键 |
| `user_id` | `BIGINT` | NOT NULL, 索引 | 关注者（`users.id`） |
| `follow_user_id` | `BIGINT` | NOT NULL, 索引 | 被关注者（`users.id`） |
| `enabled` | `BOOLEAN` | NOT NULL DEFAULT true | 关联是否启用 |
| `created_at` | `TIMESTAMPTZ` | NOT NULL | 创建时间 |
| `updated_at` | `TIMESTAMPTZ` | NOT NULL | 更新时间 |

索引（建表后单独创建）：

```sql
CREATE UNIQUE INDEX idx_user_follow_pair   ON user_follow (user_id, follow_user_id);
CREATE INDEX        idx_user_follow_follow ON user_follow (follow_user_id);
```

- `idx_user_follow_pair`：幂等去重 + "某用户关注了谁"查询
- `idx_user_follow_follow`：粉丝列表查询（谁关注了他）

> 与现有约定一致：单数表名、逻辑外键（无 DB 级 FK）、时间列 `TIMESTAMPTZ`、索引单独创建。

## 业务规则

1. **防自关注** — `user_id == follow_user_id` → `10006`
2. **幂等关注** — 已存在关系直接返回既有记录
3. **幂等取关** — 未关注时静默成功
4. **目标必须存在** — 目标 uid 不存在 → `10004`
5. **统计口径** — 粉丝数 = `count(follow_user_id = 用户)`；关注数 = `count(user_id = 用户)`
6. **分页** — `page` 从 1 开始；`page_size` 默认 20、上限 50

## API 设计

| 方法 | 路径 | 鉴权 | 说明 |
|------|------|------|------|
| POST | `/api/v1/follow/add` | ✅ Bearer | 关注某人 |
| DELETE | `/api/v1/follow/remove` | ✅ Bearer | 取关某人 |
| GET | `/api/v1/follow/stats` | ❌（可选 Bearer） | 关注统计 + 可选 `is_following` |
| GET | `/api/v1/follow/followers` | ❌ | 粉丝列表（分页） |
| GET | `/api/v1/follow/followings` | ❌ | 关注列表（分页） |

```json
// POST /follow/add
{ "target_uid": "uid_abc123" }
// → { "code": 0, "message": "ok", "data": { "id": 10, "user_id": 1, "follow_user_id": 2, "enabled": true, "created_at": "…", "updated_at": "…" } }

// GET /follow/stats（带 Bearer 时 is_following 为布尔，否则 null）
{ "user_uid": "uid_abc123" }
// → { "code": 0, "message": "ok", "data": { "user_id": 2, "follower_count": 3, "following_count": 5, "is_following": true } }

// GET /follow/followers | /followings
{ "user_uid": "uid_abc123", "page": 1, "page_size": 20 }
// → data: { "items": [ { "user_id": 2, "uid": "uid_abc123", "name": "张三", "avatar": "…" } ], "total": 3, "page": 1, "page_size": 20 }
```

## 分层落点

| 层 | 落点 | 说明 |
|----|------|------|
| `migration` | `m20260212_000000_add_user_follows` | 建 `user_follow` + 两个索引 |
| `feel_entity::follow` | `crates/feel_entity/src/follow/` | 领域模型 |
| `feel_sea_orm::follow` | `crates/feel_sea_orm/src/follow/entities/` | `UserFollowEntity` |
| `feel_storage::repo::UserFollowRepo` | `crates/feel_storage/src/repo/follow.rs` | trait + `SeaOrmUserFollowRepo` |
| `feel_storage::database::UserFollowDataBase` | `crates/feel_storage/src/database/follow.rs` | trait + `CommonUserFollowDataBase`（幂等/防自关注/分页） |
| `feel_api::model::follow` | `cmd/feel_api/src/model/follow.rs` | 请求/响应 DTO |
| `feel_api::follow` | `cmd/feel_api/src/follow/mod.rs` | handler + 路由 |
| `AppState` | `cmd/feel_api/src/lib.rs` | 注入 `user_follow_database` |

## 实施计划

**Phase 1 — MVP** ✅ 已实现

1. ✅ 迁移 `user_follow`（唯一索引 + 粉丝索引）
2. ✅ 领域模型 / SeaORM 实体
3. ✅ `UserFollowRepo` / `UserFollowDataBase`（幂等关注、防自关注、分页列表、统计）
4. ✅ API：add / remove / stats / followers / followings
5. ✅ 文档

**Phase 2 — 增强**

- 关注流 ✅ 已实现（推拉结合，见 `docs/features/feed.md`）
- 关注/取关通知（站内通知系统）
- 双向关注（互相关注）状态展示
- 关注计数 Redis 缓存（对照 `ContentCache` 模式）
- 隐私设置（关注/粉丝列表可见性）
- 批量关注 / 关注导入

## 与现有实现的关系

| 层 | 现状 |
|----|------|
| `feel_entity` | ✅ 新增 `follow` 模块（`UserFollow` / `UserFollowCreate` / `FollowStats` / `FollowPage`） |
| `feel_sea_orm` | ✅ 新增 `user_follow` 实体 + `From<Model>` 转换 |
| `feel_storage` | ✅ 沿用 Repo + DataBase 双接口模式 |
| `feel_api` | ✅ 新增 `/follow` 路由（复用 `auth_middleware` 与 `ApiResponse`） |
| `migration` | ✅ 新增 1 个迁移（1 张表 + 2 个索引） |
| 现有 API 规范 | ✅ 完全复用（JSON 请求体、统一错误码） |
