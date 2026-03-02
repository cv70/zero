use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use uuid::Uuid;
use zero_core::error::ToolError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub task_ids: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn create(&self, title: Option<String>) -> Result<Session, ToolError>;
    async fn list(&self) -> Result<Vec<Session>, ToolError>;
    async fn get(&self, id: &str) -> Result<Option<Session>, ToolError>;
    async fn update_title(&self, id: &str, title: String) -> Result<Option<Session>, ToolError>;
    async fn add_task(&self, id: &str, task_id: String) -> Result<Option<Session>, ToolError>;
    async fn remove_task(&self, id: &str, task_id: &str) -> Result<Option<Session>, ToolError>;
    async fn delete(&self, id: &str) -> Result<(), ToolError>;
}

fn now_ts() -> i64 {
    let now = SystemTime::now();
    let duration = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or(std::time::Duration::from_secs(0));
    duration.as_secs() as i64
}

pub struct InMemorySessionStore {
    sessions: Arc<RwLock<HashMap<String, Session>>>,
}

impl InMemorySessionStore {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl SessionStore for InMemorySessionStore {
    async fn create(&self, title: Option<String>) -> Result<Session, ToolError> {
        let ts = now_ts();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            title: title
                .and_then(|v| {
                    let trimmed = v.trim().to_string();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                })
                .unwrap_or_else(|| "新会话".to_string()),
            task_ids: Vec::new(),
            created_at: ts,
            updated_at: ts,
        };
        self.sessions
            .write()
            .await
            .insert(session.id.clone(), session.clone());
        Ok(session)
    }

    async fn list(&self) -> Result<Vec<Session>, ToolError> {
        let mut values: Vec<Session> = self.sessions.read().await.values().cloned().collect();
        values.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(values)
    }

    async fn get(&self, id: &str) -> Result<Option<Session>, ToolError> {
        Ok(self.sessions.read().await.get(id).cloned())
    }

    async fn update_title(&self, id: &str, title: String) -> Result<Option<Session>, ToolError> {
        let mut sessions = self.sessions.write().await;
        let Some(s) = sessions.get_mut(id) else {
            return Ok(None);
        };
        s.title = title;
        s.updated_at = now_ts();
        Ok(Some(s.clone()))
    }

    async fn add_task(&self, id: &str, task_id: String) -> Result<Option<Session>, ToolError> {
        let mut sessions = self.sessions.write().await;
        let Some(s) = sessions.get_mut(id) else {
            return Ok(None);
        };
        if !s.task_ids.iter().any(|t| t == &task_id) {
            s.task_ids.insert(0, task_id);
        }
        s.updated_at = now_ts();
        Ok(Some(s.clone()))
    }

    async fn remove_task(&self, id: &str, task_id: &str) -> Result<Option<Session>, ToolError> {
        let mut sessions = self.sessions.write().await;
        let Some(s) = sessions.get_mut(id) else {
            return Ok(None);
        };
        s.task_ids.retain(|v| v != task_id);
        s.updated_at = now_ts();
        Ok(Some(s.clone()))
    }

    async fn delete(&self, id: &str) -> Result<(), ToolError> {
        self.sessions.write().await.remove(id);
        Ok(())
    }
}

pub struct SqliteSessionStore {
    pool: SqlitePool,
}

impl SqliteSessionStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        let options = SqliteConnectOptions::new()
            .filename(path.as_ref())
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_lazy_with(options);
        Self { pool }
    }

    async fn ensure_schema(&self) -> Result<(), ToolError> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                task_ids TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("init sessions schema failed: {e}")))?;
        Ok(())
    }

    async fn persist(&self, session: &Session) -> Result<(), ToolError> {
        self.ensure_schema().await?;
        let task_ids = serde_json::to_string(&session.task_ids)
            .map_err(|e| ToolError::ExecutionFailed(format!("serialize task_ids failed: {e}")))?;
        sqlx::query(
            "INSERT INTO sessions (id, title, task_ids, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?)
             ON CONFLICT(id) DO UPDATE SET
             title=excluded.title,
             task_ids=excluded.task_ids,
             created_at=excluded.created_at,
             updated_at=excluded.updated_at",
        )
        .bind(&session.id)
        .bind(&session.title)
        .bind(task_ids)
        .bind(session.created_at)
        .bind(session.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("upsert session failed: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl SessionStore for SqliteSessionStore {
    async fn create(&self, title: Option<String>) -> Result<Session, ToolError> {
        let ts = now_ts();
        let session = Session {
            id: Uuid::new_v4().to_string(),
            title: title
                .and_then(|v| {
                    let trimmed = v.trim().to_string();
                    if trimmed.is_empty() {
                        None
                    } else {
                        Some(trimmed)
                    }
                })
                .unwrap_or_else(|| "新会话".to_string()),
            task_ids: Vec::new(),
            created_at: ts,
            updated_at: ts,
        };
        self.persist(&session).await?;
        Ok(session)
    }

    async fn list(&self) -> Result<Vec<Session>, ToolError> {
        self.ensure_schema().await?;
        let rows = sqlx::query(
            "SELECT id, title, task_ids, created_at, updated_at
             FROM sessions ORDER BY updated_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("list sessions failed: {e}")))?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows {
            let task_ids_json: String = row.get("task_ids");
            out.push(Session {
                id: row.get("id"),
                title: row.get("title"),
                task_ids: serde_json::from_str(&task_ids_json).unwrap_or_default(),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            });
        }
        Ok(out)
    }

    async fn get(&self, id: &str) -> Result<Option<Session>, ToolError> {
        self.ensure_schema().await?;
        let row = sqlx::query(
            "SELECT id, title, task_ids, created_at, updated_at FROM sessions WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ToolError::ExecutionFailed(format!("get session failed: {e}")))?;

        let Some(row) = row else {
            return Ok(None);
        };

        let task_ids_json: String = row.get("task_ids");
        Ok(Some(Session {
            id: row.get("id"),
            title: row.get("title"),
            task_ids: serde_json::from_str(&task_ids_json).unwrap_or_default(),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        }))
    }

    async fn update_title(&self, id: &str, title: String) -> Result<Option<Session>, ToolError> {
        let Some(mut session) = self.get(id).await? else {
            return Ok(None);
        };
        session.title = title;
        session.updated_at = now_ts();
        self.persist(&session).await?;
        Ok(Some(session))
    }

    async fn add_task(&self, id: &str, task_id: String) -> Result<Option<Session>, ToolError> {
        let Some(mut session) = self.get(id).await? else {
            return Ok(None);
        };
        if !session.task_ids.iter().any(|t| t == &task_id) {
            session.task_ids.insert(0, task_id);
        }
        session.updated_at = now_ts();
        self.persist(&session).await?;
        Ok(Some(session))
    }

    async fn remove_task(&self, id: &str, task_id: &str) -> Result<Option<Session>, ToolError> {
        let Some(mut session) = self.get(id).await? else {
            return Ok(None);
        };
        session.task_ids.retain(|v| v != task_id);
        session.updated_at = now_ts();
        self.persist(&session).await?;
        Ok(Some(session))
    }

    async fn delete(&self, id: &str) -> Result<(), ToolError> {
        self.ensure_schema().await?;
        sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("delete session failed: {e}")))?;
        Ok(())
    }
}
