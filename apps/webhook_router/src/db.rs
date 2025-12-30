use std::collections::HashMap;
use std::str::FromStr;

use serde_json::Value;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::{QueryBuilder, Row, Sqlite};
use uuid::Uuid;

use crate::models::{
    CreateEndpointRequest, CreateTargetRequest, DeliveryRecord, Endpoint, EventRecord, Target,
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
        sqlx::migrate!().run(&self.pool).await?;
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

    pub async fn delete_endpoint(&self, id: &str) -> Result<u64, sqlx::Error> {
        let result = sqlx::query("DELETE FROM endpoints WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
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
            deliveries: Vec::new(),
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

    pub async fn list_events(
        &self,
        endpoint_id: Option<&str>,
        page: Option<i64>,
        page_size: Option<i64>,
    ) -> Result<Vec<EventRecord>, sqlx::Error> {
        let page = page.unwrap_or(1).max(1);
        let page_size = page_size.unwrap_or(50).clamp(1, 100);
        let offset = (page - 1) * page_size;

        let (query_str, has_filter) = if let Some(_ep_id) = endpoint_id {
            (
                "SELECT id, endpoint_id, platform, title, markdown, raw, created_at
                 FROM events WHERE endpoint_id = ? ORDER BY created_at DESC LIMIT ? OFFSET ?",
                true,
            )
        } else {
            (
                "SELECT id, endpoint_id, platform, title, markdown, raw, created_at
                 FROM events ORDER BY created_at DESC LIMIT ? OFFSET ?",
                false,
            )
        };

        let rows = if has_filter {
            sqlx::query(query_str)
                .bind(endpoint_id.unwrap())
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query(query_str)
                .bind(page_size)
                .bind(offset)
                .fetch_all(&self.pool)
                .await?
        };

        let mut events = Vec::new();
        let mut event_ids = Vec::new();

        for row in rows {
            let id: String = row.get("id");
            event_ids.push(id.clone());
            events.push(EventRecord {
                id,
                endpoint_id: row.get("endpoint_id"),
                platform: row.get("platform"),
                title: row.get::<Option<String>, _>("title"),
                markdown: row.get("markdown"),
                raw: serde_json::from_str(&row.get::<String, _>("raw"))
                    .unwrap_or(Value::Null),
                created_at: row.get("created_at"),
                deliveries: Vec::new(),
            });
        }

        if event_ids.is_empty() {
            return Ok(events);
        }

        let mut builder = QueryBuilder::<Sqlite>::new(
            "SELECT d.event_id, d.target_id, t.name AS target_name, t.kind AS target_kind, d.status, d.response_code, d.error, d.created_at \
             FROM deliveries d \
             LEFT JOIN targets t ON t.id = d.target_id \
             WHERE d.event_id IN (",
        );
        {
            let mut separated = builder.separated(", ");
            for event_id in &event_ids {
                separated.push_bind(event_id);
            }
        }
        builder.push(") ORDER BY d.created_at ASC");

        let delivery_rows = builder.build().fetch_all(&self.pool).await?;
        let mut deliveries_by_event: HashMap<String, Vec<DeliveryRecord>> = HashMap::new();
        for row in delivery_rows {
            let event_id: String = row.get("event_id");
            let delivery = DeliveryRecord {
                target_id: row.get("target_id"),
                target_name: row.get::<Option<String>, _>("target_name"),
                target_kind: row.get::<Option<String>, _>("target_kind"),
                status: row.get("status"),
                response_code: row
                    .get::<Option<i64>, _>("response_code")
                    .map(|code| code as u16),
                error: row.get::<Option<String>, _>("error"),
                created_at: row.get("created_at"),
            };
            deliveries_by_event
                .entry(event_id)
                .or_default()
                .push(delivery);
        }

        for event in &mut events {
            if let Some(deliveries) = deliveries_by_event.remove(&event.id) {
                event.deliveries = deliveries;
            }
        }

        Ok(events)
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

        let events = db.list_events(None, None, None).await.expect("list events");
        assert!(!events.is_empty());

        let endpoints = db.list_endpoints().await.expect("list endpoints");
        assert_eq!(endpoints.len(), 1);

        let targets = db.list_targets(&endpoint.id).await.expect("list targets");
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].endpoint_id, endpoint.id);

        // Test delete_endpoint
        db.delete_endpoint(&endpoint.id).await.expect("delete endpoint");
        let endpoints_after = db.list_endpoints().await.expect("list endpoints");
        assert_eq!(endpoints_after.len(), 0);
    }
}
