# Label — 用户标签

## 概述

Label(标签)是**多个用户所公有**的一个标签,用于刻画用户自身的关键特征,是其他人认识用户的**快捷途径**:浏览一个用户的标签集合,即可快速了解"这个人是谁、擅长什么、关注什么"。

标签本体(名称、公共描述、备注、影响力)对所有用户共享——任何人看到"Rust 开发者"标签都指向同一个概念。用户与标签是**多对多**关系:**一个用户拥有多个标签**,构成自己的自我画像;**一个标签下对应多个用户**,按标签可以找到同类的人。用户通过**关联**使用标签,构成自己的自我画像。

在「印象」平台上,用户通过表达来描绘自我画像,而标签是最凝练的表达形式之一——每个标签都是用户自我画像上的一个"关键词"。

## 设计目标

1. **公有共享** — 标签是公有资源,多个用户共用同一个标签本体,避免重复创建、保证概念统一
2. **快捷认知** — 标签是其他人认识用户的快捷途径,因此要求短小、直观、可读
3. **备注随本体** — 标签本体携带备注,对标签含义做补充说明,所有用户一致
4. **刻画画像** — 用户的标签集合共同构成其自我画像,是用户系统的核心功能之一

## 核心概念

| 概念 | 说明 |
|------|------|
| 标签(Label) | 公有的标签本体,如"Rust 开发者""马拉松爱好者",名称全局唯一 |
| 用户标签关联(UserLabel) | 用户与标签的关联记录,表示用户与标签的多对多关系 |
| 标签集合(LabelSet) | 一个用户的全部关联,共同刻画其自我画像 |

## 领域模型

### LabelBase — 标签本体(公有)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LabelBase {
    pub id: i64,
    pub name: String,           // 全局唯一
    pub description: String,    // 公共描述
    pub remark: String,         // 标签备注(所有用户共享)
    pub influence: i64,         // 标签影响力,创建标签时确立
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 数据库主键 |
| `name` | `String` | 标签名称(全局唯一标识) |
| `description` | `String` | 标签的公共描述,帮助所有人理解该标签的含义 |
| `remark` | `String` | 标签备注,对标签含义的补充说明,所有用户共享 |
| `influence` | `i64` | 标签影响力,在创建标签时确立,之后不再变更 |
| `enabled` | `bool` | 是否启用 |
| `created_at` | `DateTime<Utc>` | 记录创建时间 |
| `updated_at` | `DateTime<Utc>` | 记录更新时间 |

**JSON 示例:**

```json
{
    "id": 1,
    "name": "Rust 开发者",
    "description": "使用 Rust 进行开发的人",
    "remark": "5 年 Rust 后端开发经验",
    "influence": 100,
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

### UserLabel — 用户标签关联

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLabel {
    pub id: i64,
    pub user_id: i64,       // 归属用户(关联 UserBase.id)
    pub label_id: i64,      // 关联 LabelBase.id
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | `i64` | 数据库主键 |
| `user_id` | `i64` | 归属用户标识,关联 `UserBase.id` |
| `label_id` | `i64` | 关联的标签本体,关联 `LabelBase.id` |
| `enabled` | `bool` | 是否启用 |
| `created_at` | `DateTime<Utc>` | 记录创建时间 |
| `updated_at` | `DateTime<Utc>` | 记录更新时间 |

> **设计说明**:`crates/feel_entity/src/label/models/mod.rs` 已包含 `LabelBase`(含 `remark` 字段)与 `UserLabel`,与本文档保持一致。

**JSON 示例:**

```json
{
    "id": 10,
    "user_id": 1,
    "label_id": 1,
    "enabled": true,
    "created_at": "2026-07-11T03:00:00Z",
    "updated_at": "2026-07-11T03:00:00Z"
}
```

### 关系模型

标签与用户是**多对多(M:N)** 关系,通过 `user_label` 关联表承载:

```
UserBase (1) ──────── (N) UserLabel (N) ──────── (1) LabelBase
  id                           user_id                    id
                               label_id

一个用户 ──拥有──► 多个标签(UserLabel)
一个标签 ──对应──► 多个用户(UserLabel)
```

- **正向(用户 → 标签)**:一个用户可以关联多个标签,这些关联构成其自我画像
- **反向(标签 → 用户)**:一个标签可以被多个用户关联,按标签即可找到使用它的用户群
- 标签本体是公有的,不属于任何单个用户
- 备注挂在标签本体上:所有用户看到同一份标签备注

## 数据模型

### 表 `label`(标签本体)

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGINT` | PK,自增 | 主键 |
| `name` | `VARCHAR` | UNIQUE | 标签名称(全局唯一) |
| `description` | `VARCHAR` | 默认空 | 公共描述 |
| `remark` | `VARCHAR` | 默认空 | 标签备注 |
| `influence` | `BIGINT` | NOT NULL | 标签影响力(创建标签时确立,之后不变) |
| `enabled` | `BOOLEAN` | NOT NULL | 是否启用 |
| `created_at` | `DATETIME` | NOT NULL | 创建时间 |
| `updated_at` | `DATETIME` | NOT NULL | 更新时间 |

### 表 `user_label`(用户标签关联)

| 列名 | 类型 | 约束 | 说明 |
|------|------|------|------|
| `id` | `BIGINT` | PK,自增 | 主键 |
| `user_id` | `BIGINT` | 索引 | 归属用户(关联 `user.id`) |
| `label_id` | `BIGINT` | 索引 | 关联标签(关联 `label.id`) |
| `enabled` | `BOOLEAN` | NOT NULL | 是否启用 |
| `created_at` | `DATETIME` | NOT NULL | 创建时间 |
| `updated_at` | `DATETIME` | NOT NULL | 更新时间 |

> 唯一约束:一个用户对同一标签只能关联一次,即 `UNIQUE(user_id, label_id)`。

## 业务规则

1. **公有性** — 标签本体是公有资源,任意用户可见、可用;不存在"私人标签"
2. **全局唯一** — 标签名全局唯一;用户添加标签时,若该标签已存在则复用本体,不重复创建
3. **备注随本体** — 备注属于标签本体,由标签本体维护,所有用户共享同一份备注
4. **数量上限** — 建议单个用户的标签关联数量上限为 20 个,防止画像被噪音稀释
5. **内容约束** — 标签名建议 1~20 个字符,不允许纯空白;备注建议不超过 100 个字符
6. **可见性** — 标签与备注默认公开,是他人认识用户的途径;不提供私密标签
7. **权威性** — 用户只能管理**自己的关联**;标签本体(含备注)不随单个用户删除而删除(公有不属于任何人)
8. **本体维护** — 标签本体的创建、禁用需经校验(如名称唯一、合法),一般不提供删除,避免影响其他已关联用户

## 核心交互

### 查看用户标签

任何用户(含未登录)可查看某用户的公开标签集合,这是"认识用户"的入口:

```
GET /api/user/{uid}/labels
```

返回该用户按创建时间排序的关联列表,每个关联内嵌标签本体信息(含备注):

```json
[
    {
        "id": 10,
        "label": { "id": 1, "name": "Rust 开发者", "remark": "5 年 Rust 后端开发经验" }
    }
]
```

### 查看标签下的用户

任何用户(含未登录)可查看某个标签下关联的所有用户,这是"按标签认识一群人"的入口(与"查看用户标签"互为反向):

```
GET /api/labels/{label_id}/users
```

返回关联了该标签的用户列表:

```json
[
    {
        "user": { "uid": "uid_abc123", "name": "张三", "avatar": "https://example.com/avatar.png" }
    }
]
```

### 添加标签

用户为画像添加一个标签:

```
POST /api/user/{uid}/labels
Body: { "label_id": 1 }
```

若标签不存在,也可按名称创建本体后再关联:

```
Body: { "name": "马拉松爱好者", "description": "", "influence": 50, "remark": "" }
```

校验通过后创建 `UserLabel` 关联(必要时先创建 `LabelBase`),返回完整关联数据。

### 修改标签备注

更新标签本体的备注(需具备标签维护权限):

```
PUT /api/labels/{label_id}
Body: { "remark": "跑过 5 个全马" }
```

### 解除关联

用户移除自己画像上的某个标签(仅解除关联,不删除公有标签本体):

```
DELETE /api/user/{uid}/labels/{user_label_id}
```

## 与现有实现的关系

| 层 | 现状 | 本 feature 所需 |
|----|------|-----------------|
| `feel_entity::label` | 已有 `LabelBase`(含 `remark` 字段)与 `UserLabel` | — |
| `feel_sea_orm` | 已有 `label`、`user_label` ORM 实体(含 `From<Model>` 转换) | — |
| `feel_storage` | 已有 `LabelRepo` + `SeaOrmLabelRepo`(统一操作 `LabelBase` 与 `UserLabel`,按 `user_id` 过滤) | — |
| `feel_api` | 暂无 label 相关 handler | 新增标签增删改查接口,以及"查看标签下的用户"接口 |
| `migration` | 已有 `label`、`user_label` 建表迁移 | — |

## 设计说明

1. **多对多设计** — 标签与用户是"多对多"关系,通过 `user_label` 关联表承载;标签本体与关联分离,实现"公有标签本体 + 用户关联"
2. **备注随本体** — 备注放在 `LabelBase` 上,对所有用户一致;`UserLabel` 仅承载"用户×标签"关联,不携带个性化内容
3. **按 `user_id` 关联用户** — `UserLabel.user_id` 关联 `UserBase.id`,以数据库主键做跨表关联
4. **公开只读** — 标签的核心价值在于被他人浏览,因此不设私密状态,减少权限分支
5. **本体只增不删** — 标签本体公有,删除会影响其他关联用户,故只支持创建/禁用,不支持物理删除
6. **影响力创建时确立** — `influence` 在创建标签时由创建者确立(如给出初始影响力值),一经创建不再随关联变动而变更,作为标签的固有属性
7. **自我刻画** — 标签由用户自己选择,是"通过表达描绘自我画像"的一种轻量表达;更重的表达走文章、评论等其他功能

## 未来扩展

- **标签排序/置顶** — 支持用户调整关联顺序,突出核心特征
- **标签热度** — 统计标签被关联的用户数,可作为标签的展示参考(与创建时确立的 `influence` 并存,两者可分别用于不同排序场景)
- **系统标签推荐** — 基于用户文章/行为推荐候选标签,降低创建成本
- **标签认可** — 其他用户可对"用户×标签"表达认可(如"赞同"),增强标签的社交可信度
- **画像主页** — 以标签为核心的用户主页卡片,作为他人认识用户的一站式入口
