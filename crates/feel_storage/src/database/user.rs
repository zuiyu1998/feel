use async_trait::async_trait;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};

use crate::{Result, cache::user::UserCache, repo::UserRepo};
use feel_entity::prelude::*;

/// Claims stored in a JWT token for authenticated users.
/// Returned by [`UserDataBase::parse_token`] for inspection/verification.
#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct TokenClaims {
    /// Subject — user uid
    pub sub: String,
    /// Issued at (UNIX timestamp)
    pub iat: usize,
    /// Expiration (UNIX timestamp)
    pub exp: usize,
}

pub struct CommonUserDataBase {
    user_repo: Box<dyn UserRepo>,
    user_cache: Box<dyn UserCache>,
    jwt_secret: String,
}

impl CommonUserDataBase {
    pub fn new<T: UserRepo>(
        user_repo: T,
        user_cache: Box<dyn UserCache>,
        jwt_secret: &str,
    ) -> Self {
        CommonUserDataBase {
            user_repo: Box::new(user_repo),
            user_cache,
            jwt_secret: jwt_secret.to_string(),
        }
    }
}

#[async_trait]
impl UserDataBase for CommonUserDataBase {
    async fn register(&self, register: &UserRegister) -> crate::Result<UserBase> {
        self.user_repo.register(register).await
    }

    async fn unregister(&self, user_id: i64) -> crate::Result<UserBase> {
        self.user_repo.unregister(user_id).await
    }

    async fn login(&self, login: &UserLogin) -> crate::Result<LoginResult> {
        // 1. Authenticate via UserRepo — returns user data on success
        let user_base = self.user_repo.login(login).await?;

        // 2. Generate JWT token with user uid as subject, 7-day expiry
        let token = UserDataBase::generate_token(self, &user_base.uid)?;

        // 3. Cache the authenticated user's data for fast subsequent access
        self.user_cache.set_user_base(&user_base).await?;

        Ok(LoginResult { token, user_base })
    }

    async fn get_user(&self, uid: &str) -> crate::Result<UserBase> {
        // 1. Try cache first (keyed by numeric id — requires uid→id mapping, skip for now)
        // 2. Fallback to repo lookup
        self.user_repo
            .find_by_uid(uid)
            .await?
            .ok_or_else(|| crate::Error::Authentication(format!("User {} not found", uid)))
    }

    fn generate_token(&self, uid: &str) -> crate::Result<String> {
        let now = Utc::now();
        let claims = TokenClaims {
            sub: uid.to_string(),
            iat: now.timestamp() as usize,
            exp: (now + Duration::hours(24 * 7)).timestamp() as usize,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| crate::Error::Authentication(format!("Failed to generate token: {}", e)))
    }

    fn parse_token(&self, token: &str) -> crate::Result<TokenClaims> {
        jsonwebtoken::decode::<TokenClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| crate::Error::Authentication(format!("Failed to parse token: {}", e)))
    }

    fn logout(&self, _user_id: u32) -> crate::Result<()> {
        todo!()
    }

    fn update(&self, _update: &UserUpdate) -> crate::Result<UserBase> {
        todo!()
    }
}

#[async_trait]
pub trait UserDataBase: 'static + Send + Sync {
    ///用户系统注册用户
    async fn register(&self, register: &UserRegister) -> Result<UserBase>;
    ///用户系统注销用户
    async fn unregister(&self, user_id: i64) -> Result<UserBase>;
    ///用户登录系统
    async fn login(&self, login: &UserLogin) -> Result<LoginResult>;
    ///用户登出系统
    fn logout(&self, user_id: u32) -> Result<()>;
    ///用户系统更改个人信息
    fn update(&self, update: &UserUpdate) -> Result<UserBase>;

    // -- JWT token management --

    /// Generate a JWT token for the given user uid.
    /// The token embeds the uid, issue time, and a 7-day expiration.
    fn generate_token(&self, uid: &str) -> Result<String>;

    /// Parse and validate a JWT token, returning the embedded claims.
    /// Returns `Authentication` error if the token is invalid or expired.
    fn parse_token(&self, token: &str) -> Result<TokenClaims>;

    // -- User query --

    /// 根据用户 UID 获取用户基础信息
    async fn get_user(&self, uid: &str) -> Result<UserBase>;
}
