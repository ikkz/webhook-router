use std::str::FromStr;

use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::Row;
use uuid::Uuid;

use crate::models::{
    CreateEndpointRequest, CreateTargetRequest, Endpoint, EventRecord, Target,
    UpdateEndpointRequest, UemEvent,
};

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn connect(path: &str) -> Result<Self, sqlx::Error> {
        let (options, in_memory) = if path == ":memory:" {
            (SqliteConnectOptions::from_str("sqlite::memory:")?, true)
        } else if path.starts_with("sqlite:") {
            let in_memory = path.contains("memory");
            (SqliteConnectOptions::from_str(path)?, in_memory)
        } else {
            (
                SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(true),
                false,
            )
        };
        let pool = if in_memory {
            SqlitePoolOptions::new()
                .max_connections(1)
                .connect_with(options)
                .await?
        } else {
            SqlitePoolOptions::new().connect_with(options).await?
        };
        let db = Self { pool };
        db.init().await?;
        Ok(db)
    }

    async fn init(&self) -> Result<(), sqlx::Error> {
        // Drop tables if they exist to force schema update since we are changing it
        // In a real app we would use migrations, but for this dev stage we drop.
        // Actually, user agreed to delete the DB file, but running this command helps if they didn't.
        // But for safety let's just create if not exists with new schema, 
        // assuming the user deletes the file. If not, it might error or have old columns.
        // I'll rely on "delete webhook_router.db" step.
        
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS endpoints (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                banner TEXT,
                footer TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS targets (
                id TEXT PRIMARY KEY,
                endpoint_id TEXT NOT NULL,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                url TEXT NOT NULL,
                headers TEXT,
                created_at INTEGER NOT NULL,
                FOREIGN KEY(endpoint_id) REFERENCES endpoints(id) ON DELETE CASCADE
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                endpoint_id TEXT NOT NULL,
                platform TEXT NOT NULL,
                title TEXT,
                markdown TEXT NOT NULL,
                raw TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS deliveries (
                id TEXT PRIMARY KEY,
                event_id TEXT NOT NULL,
                target_id TEXT NOT NULL,
                status TEXT NOT NULL,
                response_code INTEGER,
                error TEXT,
                created_at INTEGER NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn create_target(&self, endpoint_id: &str, req: CreateTargetRequest) -> Result<Target, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_timestamp();
        let headers = req.headers.map(|value| value.to_string());
        sqlx::query(
            "INSERT INTO targets (id, endpoint_id, name, kind, url, headers, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(endpoint_id)
        .bind(&req.name)
        .bind(&req.kind)
        .bind(&req.url)
        .bind(headers.clone())
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(Target {
            id,
            endpoint_id: endpoint_id.to_string(),
            name: req.name,
            kind: req.kind,
            url: req.url,
            headers: headers.and_then(|raw| serde_json::from_str(&raw).ok()),
            created_at,
        })
    }

    pub async fn list_targets(&self, endpoint_id: &str) -> Result<Vec<Target>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, endpoint_id, name, kind, url, headers, created_at
             FROM targets WHERE endpoint_id = ? ORDER BY created_at DESC",
        )
        .bind(endpoint_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Target {
                id: row.get("id"),
                endpoint_id: row.get("endpoint_id"),
                name: row.get("name"),
                kind: row.get("kind"),
                url: row.get("url"),
                headers: row
                    .get::<Option<String>, _>("headers")
                    .and_then(|raw| serde_json::from_str(&raw).ok()),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    pub async fn delete_target(&self, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM targets WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    #[allow(dead_code)]
    pub async fn get_target(&self, id: &str) -> Result<Option<Target>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, endpoint_id, name, kind, url, headers, created_at
             FROM targets WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Target {
            id: row.get("id"),
            endpoint_id: row.get("endpoint_id"),
            name: row.get("name"),
            kind: row.get("kind"),
            url: row.get("url"),
            headers: row
                .get::<Option<String>, _>("headers")
                .and_then(|raw| serde_json::from_str(&raw).ok()),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn create_endpoint(
        &self,
        req: CreateEndpointRequest,
    ) -> Result<Endpoint, sqlx::Error> {
        let id = Uuid::new_v4().to_string();
        let created_at = now_timestamp();

        sqlx::query(
            "INSERT INTO endpoints (id, name, banner, footer, created_at)
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(&req.name)
        .bind(&req.banner)
        .bind(&req.footer)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(Endpoint {
            id,
            name: req.name,
            banner: req.banner,
            footer: req.footer,
            created_at,
        })
    }

    pub async fn list_endpoints(&self) -> Result<Vec<Endpoint>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, name, banner, footer, created_at
             FROM endpoints ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| Endpoint {
                id: row.get("id"),
                name: row.get("name"),
                banner: row.get("banner"),
                footer: row.get("footer"),
                created_at: row.get("created_at"),
            })
            .collect())
    }

    pub async fn get_endpoint(&self, id: &str) -> Result<Option<Endpoint>, sqlx::Error> {
        let row = sqlx::query(
            "SELECT id, name, banner, footer, created_at
             FROM endpoints WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|row| Endpoint {
            id: row.get("id"),
            name: row.get("name"),
            banner: row.get("banner"),
            footer: row.get("footer"),
            created_at: row.get("created_at"),
        }))
    }

    pub async fn update_endpoint(
        &self,
        id: &str,
        req: UpdateEndpointRequest,
    ) -> Result<Option<Endpoint>, sqlx::Error> {
        let mut endpoint = match self.get_endpoint(id).await? {
            Some(endpoint) => endpoint,
            None => return Ok(None),
        };

        if let Some(name) = req.name {
            endpoint.name = name;
        }
        if req.banner.is_some() {
            endpoint.banner = req.banner;
        }
        if req.footer.is_some() {
            endpoint.footer = req.footer;
        }

        sqlx::query(
            "UPDATE endpoints SET name = ?, banner = ?, footer = ? WHERE id = ?",
        )
        .bind(&endpoint.name)
        .bind(&endpoint.banner)
        .bind(&endpoint.footer)
        .bind(&endpoint.id)
        .execute(&self.pool)
        .await?;

        Ok(Some(endpoint))
    }

    pub async fn insert_event(
        &self,
        endpoint_id: &str,
        platform: &str,
        event: &UemEvent,
    ) -> Result<EventRecord, sqlx::Error> {
        let created_at = now_timestamp();
        let raw = event.raw.to_string();

        sqlx::query(
            "INSERT INTO events (id, endpoint_id, platform, title, markdown, raw, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&event.id)
        .bind(endpoint_id)
        .bind(platform)
        .bind(event.title.as_deref())
        .bind(&event.markdown)
        .bind(&raw)
        .bind(created_at)
        .execute(&self.pool)
        .await?;

        Ok(EventRecord {
            id: event.id.clone(),
            endpoint_id: endpoint_id.to_string(),
            platform: platform.to_string(),
            title: event.title.clone(),
            markdown: event.markdown.clone(),
            raw: event.raw.clone(),
            created_at,
        })
    }

    pub async fn insert_delivery(
        &self,
        event_id: &str,
        target_id: &str,
        status: &str,
        response_code: Option<u16>,
        error: Option<String>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO deliveries (id, event_id, target_id, status, response_code, error, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(Uuid::new_v4().to_string())
        .bind(event_id)
        .bind(target_id)
        .bind(status)
        .bind(response_code.map(i64::from))
        .bind(error)
        .bind(now_timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_events(&self) -> Result<Vec<EventRecord>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT id, endpoint_id, platform, title, markdown, raw, created_at
             FROM events ORDER BY created_at DESC LIMIT 100",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| EventRecord {
                id: row.get("id"),
                endpoint_id: row.get("endpoint_id"),
                platform: row.get("platform"),
                title: row.get::<Option<String>, _>("title"),
                markdown: row.get("markdown"),
                raw: serde_json::from_str(&row.get::<String, _>("raw"))
                    .unwrap_or(Value::Null),
                created_at: row.get("created_at"),
            })
            .collect())
    }
}

fn now_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{CreateEndpointRequest, CreateTargetRequest, UemEvent};
    use serde_json::json;

    #[tokio::test]
    async fn sqlite_in_memory_flow() {
        let db = Db::connect(":memory:").await.expect("connect");

        let endpoint = db
            .create_endpoint(CreateEndpointRequest {
                name: "demo".to_string(),
                banner: None,
                footer: None,
            })
            .await
            .expect("create endpoint");

        let target = db
            .create_target(&endpoint.id, CreateTargetRequest {
                name: "Slack".to_string(),
                kind: "slack".to_string(),
                url: "https://example.com/hook".to_string(),
                headers: Some(json!({"X-Test": "yes"})),
            })
            .await
            .expect("create target");

        let event = UemEvent {
            id: "evt-1".to_string(),
            source: "slack".to_string(),
            timestamp: 123,
            title: Some("message".to_string()),
            markdown: "hello".to_string(),
            raw: json!({"text": "hello"}),
            meta: json!({}),
        };

        let record = db
            .insert_event(&endpoint.id, "slack", &event)
            .await
            .expect("insert event");

        assert_eq!(record.endpoint_id, endpoint.id);
        assert_eq!(record.markdown, "hello");

        db.insert_delivery(&event.id, &target.id, "sent", Some(200), None)
            .await
            .expect("insert delivery");

        let events = db.list_events().await.expect("list events");
        assert!(!events.is_empty());

        let endpoints = db.list_endpoints().await.expect("list endpoints");
        assert_eq!(endpoints.len(), 1);

        let targets = db.list_targets(&endpoint.id).await.expect("list targets");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].endpoint_id, endpoint.id);
    }
}
