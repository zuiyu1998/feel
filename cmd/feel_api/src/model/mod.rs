//! API 专用的请求/响应数据模型（DTO）。
//!
//! 与 `feel_entity` 中的领域模型不同，此模块中的类型专注于：
//! - HTTP JSON 序列化/反序列化（通过 serde）
//! - 输入校验和字段映射
//! - API 版本兼容性

pub mod response;
pub mod user;
