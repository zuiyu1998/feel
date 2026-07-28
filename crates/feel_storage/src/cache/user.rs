use crate::Result;
use async_trait::async_trait;
use feel_entity::prelude::*;
use redis::{Client, RedisResult};
use serde_json;

const REDIS_KEY_USERS: &str = "users";

pub struct CommonUserCache {
    client: Client,
}

impl CommonUserCache {
    pub fn new(client: Client) -> Self {
        CommonUserCache { client }
    }

    async fn get_connection(&self) -> RedisResult<redis::aio::MultiplexedConnection> {
        self.client.get_multiplexed_tokio_connection().await
    }
}

#[async_trait]
impl UserCache for CommonUserCache {
    async fn set_user_base(&self, user_base: &UserBase) -> Result<()> {
        let mut conn = self.get_connection().await?;
        let json = serde_json::to_string(user_base)?;
        redis::cmd("HSET")
            .arg(REDIS_KEY_USERS)
            .arg(&user_base.uid)
            .arg(json)
            .query_async::<_, ()>(&mut conn)
            .await?;
        Ok(())
    }

    async fn get_user_base(&self, user_id: &str) -> Result<Option<UserBase>> {
        let mut conn = self.get_connection().await?;
        let json: Option<String> = redis::cmd("HGET")
            .arg(REDIS_KEY_USERS)
            .arg(user_id)
            .query_async(&mut conn)
            .await?;

        match json {
            Some(json_str) => {
                let user_base = serde_json::from_str(&json_str)?;
                Ok(Some(user_base))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
pub trait UserCache: 'static + Send + Sync {
    ///设置用户基础信息到缓存
    async fn set_user_base(&self, user_base: &UserBase) -> Result<()>;
    ///从缓存获取用户基础信息
    async fn get_user_base(&self, user_id: &str) -> Result<Option<UserBase>>;
}
