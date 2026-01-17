// Blindodon - An accessibility-first Mastodon client
// Copyright (C) 2025 Blindodon Contributors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Cache module for local data storage
//!
//! Uses SQLite for persistent caching of posts, users, and other data.

use anyhow::Result;
use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::models::StoredAccount;

/// Cache manager for local data storage
pub struct CacheManager {
    pool: SqlitePool,
}

impl CacheManager {
    /// Create a new cache manager
    pub async fn new() -> Result<Self> {
        let db_path = get_db_path();

        // Ensure the directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

        info!("Opening cache database at {}", db_path.display());

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let manager = Self { pool };
        manager.init_schema().await?;

        Ok(manager)
    }

    /// Initialize the database schema
    async fn init_schema(&self) -> Result<()> {
        debug!("Initializing cache schema");

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS posts (
                id TEXT PRIMARY KEY,
                account_id TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL,
                data TEXT NOT NULL,
                cached_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_posts_account ON posts(account_id);
            CREATE INDEX IF NOT EXISTS idx_posts_created ON posts(created_at);

            CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                acct TEXT NOT NULL,
                display_name TEXT,
                data TEXT NOT NULL,
                cached_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE INDEX IF NOT EXISTS idx_users_acct ON users(acct);

            CREATE TABLE IF NOT EXISTS accounts (
                id TEXT PRIMARY KEY,
                instance_url TEXT NOT NULL,
                username TEXT NOT NULL,
                access_token TEXT NOT NULL,
                refresh_token TEXT,
                data TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                last_used_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
                is_default INTEGER NOT NULL DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS timeline_positions (
                timeline_id TEXT PRIMARY KEY,
                last_read_id TEXT,
                scroll_position INTEGER DEFAULT 0,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Cache schema initialized");

        Ok(())
    }

    /// Get the database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Clean up old cached data
    pub async fn cleanup(&self, max_age_days: u32) -> Result<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM posts
            WHERE cached_at < datetime('now', '-' || ? || ' days')
            "#,
        )
        .bind(max_age_days)
        .execute(&self.pool)
        .await?;

        let deleted = result.rows_affected();
        if deleted > 0 {
            info!("Cleaned up {} old cached posts", deleted);
        }

        Ok(deleted)
    }

    // ===== ACCOUNT CRUD METHODS =====

    /// Save or update an account in the database
    pub async fn save_account(&self, account: &StoredAccount) -> Result<()> {
        let data = serde_json::to_string(account)?;

        sqlx::query(
            r#"
            INSERT INTO accounts (id, instance_url, username, access_token, refresh_token, data, created_at, last_used_at, is_default)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                data = excluded.data,
                last_used_at = excluded.last_used_at,
                is_default = excluded.is_default
            "#,
        )
        .bind(&account.id)
        .bind(&account.instance_url)
        .bind(&account.username)
        .bind(&account.access_token)
        .bind(&account.refresh_token)
        .bind(&data)
        .bind(account.added_at.to_rfc3339())
        .bind(account.last_used_at.to_rfc3339())
        .bind(account.is_default)
        .execute(&self.pool)
        .await?;

        info!("Saved account {} ({})", account.acct, account.id);
        Ok(())
    }

    /// Get all saved accounts
    pub async fn get_accounts(&self) -> Result<Vec<StoredAccount>> {
        let rows: Vec<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT data, access_token, refresh_token FROM accounts ORDER BY last_used_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        let accounts: Vec<StoredAccount> = rows
            .into_iter()
            .filter_map(|(data, access_token, refresh_token)| {
                let mut account: StoredAccount = serde_json::from_str(&data).ok()?;
                // Restore sensitive fields that were skipped during serialization
                account.access_token = access_token;
                account.refresh_token = refresh_token;
                Some(account)
            })
            .collect();

        Ok(accounts)
    }

    /// Get the default (or most recently used) account
    pub async fn get_default_account(&self) -> Result<Option<StoredAccount>> {
        // First try to get account marked as default
        let row: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT data, access_token, refresh_token FROM accounts WHERE is_default = 1 LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        // If no default, get most recently used
        let row = match row {
            Some(r) => Some(r),
            None => {
                sqlx::query_as(
                    "SELECT data, access_token, refresh_token FROM accounts ORDER BY last_used_at DESC LIMIT 1",
                )
                .fetch_optional(&self.pool)
                .await?
            }
        };

        match row {
            Some((data, access_token, refresh_token)) => {
                let mut account: StoredAccount = serde_json::from_str(&data)?;
                account.access_token = access_token;
                account.refresh_token = refresh_token;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    /// Get account by ID
    pub async fn get_account(&self, account_id: &str) -> Result<Option<StoredAccount>> {
        let row: Option<(String, String, Option<String>)> = sqlx::query_as(
            "SELECT data, access_token, refresh_token FROM accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((data, access_token, refresh_token)) => {
                let mut account: StoredAccount = serde_json::from_str(&data)?;
                account.access_token = access_token;
                account.refresh_token = refresh_token;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    /// Delete an account
    pub async fn delete_account(&self, account_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(&self.pool)
            .await?;

        info!("Deleted account {}", account_id);
        Ok(())
    }

    /// Set account as default and update last_used_at
    pub async fn set_default_account(&self, account_id: &str) -> Result<()> {
        // Clear all default flags
        sqlx::query("UPDATE accounts SET is_default = 0")
            .execute(&self.pool)
            .await?;

        // Set the new default
        sqlx::query(
            "UPDATE accounts SET is_default = 1, last_used_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(account_id)
        .execute(&self.pool)
        .await?;

        debug!("Set default account to {}", account_id);
        Ok(())
    }

    // ===== SETTINGS CRUD METHODS =====

    /// Get a setting value
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT value FROM settings WHERE key = ?")
                .bind(key)
                .fetch_optional(&self.pool)
                .await?;

        Ok(row.map(|(v,)| v))
    }

    /// Set a setting value
    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO settings (key, value, updated_at)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                updated_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(key)
        .bind(value)
        .execute(&self.pool)
        .await?;

        debug!("Set setting {} = {}", key, value);
        Ok(())
    }

    /// Delete a setting
    pub async fn delete_setting(&self, key: &str) -> Result<()> {
        sqlx::query("DELETE FROM settings WHERE key = ?")
            .bind(key)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get all settings as a map
    pub async fn get_all_settings(&self) -> Result<HashMap<String, String>> {
        let rows: Vec<(String, String)> =
            sqlx::query_as("SELECT key, value FROM settings")
                .fetch_all(&self.pool)
                .await?;

        Ok(rows.into_iter().collect())
    }
}

/// Get the database file path
fn get_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Blindodon")
        .join("cache.db")
}
