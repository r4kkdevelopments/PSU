use chrono::Utc;
use postgres::rows::Row;
use serde::{Deserialize, Serialize};

use super::account_services;
#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub discord_id: Option<String>,
    pub address: Option<String>,
    pub country_id: Option<i64>,
    pub role_id: Option<i64>,
    pub birthday: Option<chrono::NaiveDate>,
    pub last_login: Option<chrono::DateTime<Utc>>,
    pub status: Option<String>,
    pub two_factor_country_code: Option<i64>,
    pub two_factor_phone: Option<i64>,
    pub two_factor_options: Option<String>,
    pub email_verified_at: Option<chrono::DateTime<Utc>>,
    pub remember_token: Option<String>,
    pub created_at: Option<chrono::DateTime<Utc>>,
    pub updated_at: Option<chrono::DateTime<Utc>>,
    pub announcements_last_read_at: Option<chrono::DateTime<Utc>>,
    pub api_enabled: Option<String>,
    pub api_key: Option<String>,
    pub discord_ids: Option<Vec<String>>,
    pub discord_username: Option<String>,
    pub discord_avatar: Option<String>,
}

// User Struct without classified information
#[derive(Debug, Deserialize, Serialize)]
pub struct SafeUser {
    pub id: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar: Option<String>,
    pub role_id: Option<i64>,
    pub last_login: Option<chrono::DateTime<Utc>>,
    pub status: Option<String>,
    pub announcements_last_read_at: Option<chrono::DateTime<Utc>>,
    pub api_enabled: Option<String>,
    pub api_key: Option<String>,
    pub has_premium: Option<String>,
    pub discord_ids: Option<Vec<String>>,
    pub discord_username: Option<String>,
    pub discord_avatar: Option<String>,
    pub discord_id: Option<String>,
}

impl User {
    pub fn get_safe_user(self: &Self, conn: &crate::MainPGDatabase) -> SafeUser {
        let has_premium = account_services::has_premium(&self.id, &conn);
        SafeUser {
            id: self.id.clone(),
            email: self.email.clone(),
            username: self.username.clone(),
            first_name: self.first_name.clone(),
            last_name: self.last_name.clone(),
            avatar: self.avatar.clone(),
            role_id: self.role_id.clone(),
            last_login: self.last_login.clone(),
            status: self.status.clone(),
            announcements_last_read_at: self.announcements_last_read_at.clone(),
            api_enabled: self.api_enabled.clone(),
            api_key: self.api_key.clone(),
            discord_ids: self.discord_ids.clone(),
            discord_username: self.discord_username.clone(),
            discord_avatar: self.discord_avatar.clone(),
            discord_id: self.discord_id.clone(),
            has_premium: has_premium,
        }
    }
}

pub fn row_to_user(user_row: &Row) -> User {
    User {
        id: user_row.get("ID"),
        email: user_row.get("email"),
        username: user_row.get("username"),
        password: user_row.get("password"),
        first_name: user_row.get("first_name"),
        last_name: user_row.get("last_name"),
        phone: user_row.get("phone"),
        avatar: user_row.get("avatar"),
        discord_id: user_row.get("discord_id"),
        address: user_row.get("address"),
        country_id: user_row.get("country_id"),
        role_id: user_row.get("role_id"),
        birthday: user_row.get("birthday"),
        last_login: user_row.get("last_login"),
        status: user_row.get("status"),
        two_factor_country_code: user_row.get("two_factor_country_code"),
        two_factor_phone: user_row.get("two_factor_phone"),
        two_factor_options: user_row.get("two_factor_options"),
        email_verified_at: user_row.get("email_verified_at"),
        remember_token: user_row.get("remember_token"),
        created_at: user_row.get("created_at"),
        updated_at: user_row.get("updated_at"),
        announcements_last_read_at: user_row.get("announcements_last_read_at"),
        api_enabled: user_row.get("api_enabled"),
        api_key: user_row.get("api_key"),
        discord_ids: user_row.get("discord_ids"),
        discord_username: user_row.get("discord_username"),
        discord_avatar: user_row.get("discord_avatar"),
    }
}
