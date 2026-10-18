# feel_core — 核心公共依赖

## 概述

`feel_core` 是 feel 平台的**核心基础 crate**，位于 `crates/feel_core/`。其职责单一：集中 re-export workspace 共同依赖的第三方库，保证各 crate 使用同一版本、同一特性配置，避免版本漂移。

## 模块位置

```
crates/feel_core/src/
└── lib.rs   # re-export chrono / tokio
```

## 导出内容

```rust
pub use chrono;
pub use tokio;
```

| 导出 | 说明 |
|------|------|
| `feel_core::chrono` | 日期时间库（`DateTime<Utc>`、`Local::now()` 等） |
| `feel_core::tokio` | 异步运行时（`#[tokio::main]`、`tokio::spawn` 等） |

其他 crate 通过 `feel_core::chrono` 访问日期时间类型（如 `feel_entity` 的领域模型），而非在各自 `Cargo.toml` 直接依赖 `chrono`，从而统一版本。

## 使用示例

```rust
// 在消费方 crate 的 Cargo.toml 中声明 feel_core 依赖后：
use feel_core::chrono::{DateTime, Utc};

pub struct Foo {
    pub created_at: DateTime<Utc>,
}
```

- `feel_entity`：`use feel_core::chrono::{DateTime, Utc};`（领域模型的 `created_at` / `updated_at`）
- `feel_api`（`main.rs` 等）：异步运行入口使用 `tokio`

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| `chrono` 导出 | ✅ | 被 `feel_entity` 使用 |
| `tokio` 导出 | ✅ | 异步运行时 |
| 业务逻辑 | — | 本 crate 不含业务代码，仅做依赖统一 |

## 未来扩展

随着公共依赖增多（如 `serde` 辅助、日志初始化等），可继续在此统一导出，保持"一处升级、全仓生效"。
