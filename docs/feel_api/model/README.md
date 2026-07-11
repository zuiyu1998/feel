# feel_api Model — API 数据模型（DTO）

## 概述

`model` 模块定义 API 专用的请求/响应数据模型（DTO，Data Transfer Object），位于 `cmd/feel_api/src/model/`。

与 `feel_entity` 中的领域模型不同，此模块中的类型专注于：

- **HTTP JSON 序列化/反序列化**（通过 `serde`）
- **输入校验**和字段映射
- **API 版本兼容性**（领域模型变化时 DTO 可保持稳定）

## 模块结构

```
cmd/feel_api/src/model/
├── mod.rs    # 模块声明
└── user.rs   # 用户 API DTO
```

## 设计原则

| 关注点 | 领域模型（feel_entity） | API DTO（feel_api::model） |
|--------|------------------------|---------------------------|
| 职责 | 业务逻辑与持久化 | HTTP 序列化/反序列化 |
| 字段 | 完整业务字段 | 按接口裁剪 |
| 类型 | 使用领域类型（如 `DateTime<Utc>`） | 使用可序列化类型（如 `String`） |
| 校验 | 业务不变量 | 输入格式校验 |

当前 API handler 直接使用 `feel_entity` 的领域类型，后续可按需迁移到此模块，实现 HTTP 层与领域层的解耦。
