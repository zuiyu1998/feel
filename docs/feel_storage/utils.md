# utils — 存储层工具函数

## 概述

`crates/feel_storage/src/utils.rs` 提供与存储层相关的公共工具函数。当前仅包含密码哈希工具，供 `SeaOrmUserRepo` 在注册时使用。

## 模块位置

```
crates/feel_storage/src/
├── lib.rs        # 模块声明（pub mod utils）
├── error.rs
├── utils.rs      # 工具函数 ←
├── cache/        # 缓存
├── database/     # 数据访问网关
└── repo/         # 持久化接口与实现
```

## 函数

### `get_passwod_hash`

> 注：函数名拼写为 `get_passwod_hash`（源码原文，未改动）。

```rust
pub fn get_passwod_hash(password: &str) -> (String, String)
```

使用 **Argon2**（默认参数）对明文密码加盐哈希：

1. 用操作系统随机源生成随机盐 `SaltString`（`argon2::password_hash::rand_core::OsRng`）
2. 计算 `password_hash`
3. 返回元组 `(salt, hash)`

| 返回值 | 说明 | 写入位置 |
|--------|------|----------|
| `.0` | 盐字符串（base64） | `user_credentials.encryption_key` |
| `.1` | Argon2 哈希串 | `user_credentials.encrypted_data` |

## 使用方式

`SeaOrmUserRepo::register` 中调用：

```rust
let (salt, password_hash) = get_passwod_hash(&register.data);
// ... 构造 UserCredentialsActiveModel：
//     encryption_key = Set(salt)
//     encrypted_data = Set(password_hash)
```

登录校验时用 `Argon2::default().verify_password` 对照凭据中的 PHC 哈希串（含盐）校验候选密码。

## 当前状态

| 项 | 状态 | 说明 |
|----|------|------|
| 密码哈希（Argon2） | ✅ | 注册流程使用 |
| 密码校验 | ✅ | 登录流程使用（重算比对） |

## 相关文档

- [SeaOrmUserRepo](SeaOrmUserRepo.md) — `register` / `login` 流程
- [UserCredentialsEntity](../feel_sea_orm/UserCredentialsEntity.md) — 凭据表结构
