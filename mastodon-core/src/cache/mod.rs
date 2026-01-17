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
use std::path::PathBuf;
use tracing::{info, debug};

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
}

/// Get the database file path
fn get_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Blindodon")
        .join("cache.db")
}
