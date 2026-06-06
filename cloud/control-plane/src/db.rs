use std::path::Path;

use anyhow::Context;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct TenantRow {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct AgentRow {
    pub id: String,
    pub tenant_id: String,
    pub name: String,
    pub provider: String,
    pub service_name: String,
    pub engine_token: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub provider: String,
    #[serde(default)]
    pub api_key: Option<String>,
}

pub async fn init(db_path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let url = format!("sqlite:{}?mode=rwc", db_path.display());
    let pool = SqlitePool::connect(&url).await.context("connect sqlite")?;
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tenants (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            namespace TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY NOT NULL,
            tenant_id TEXT NOT NULL,
            name TEXT NOT NULL,
            provider TEXT NOT NULL,
            service_name TEXT NOT NULL,
            engine_token TEXT NOT NULL,
            created_at TEXT NOT NULL,
            FOREIGN KEY (tenant_id) REFERENCES tenants(id)
        );
        "#,
    )
    .execute(&pool)
    .await?;
    Ok(pool)
}

pub async fn insert_tenant(pool: &SqlitePool, name: &str) -> anyhow::Result<TenantRow> {
    let id = Uuid::new_v4().to_string();
    let slug = slugify(name);
    let namespace = format!("tenant-{slug}");
    let created_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO tenants (id, name, namespace, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(name)
    .bind(&namespace)
    .bind(&created_at)
    .execute(pool)
    .await?;
    Ok(TenantRow {
        id,
        name: name.to_string(),
        namespace,
        created_at,
    })
}

pub async fn list_tenants(pool: &SqlitePool) -> anyhow::Result<Vec<TenantRow>> {
    Ok(sqlx::query_as::<_, TenantRow>("SELECT id, name, namespace, created_at FROM tenants ORDER BY created_at")
        .fetch_all(pool)
        .await?)
}

pub async fn get_tenant(pool: &SqlitePool, tenant_id: &str) -> anyhow::Result<Option<TenantRow>> {
    Ok(
        sqlx::query_as::<_, TenantRow>(
            "SELECT id, name, namespace, created_at FROM tenants WHERE id = ?",
        )
        .bind(tenant_id)
        .fetch_optional(pool)
        .await?,
    )
}

pub async fn insert_agent(
    pool: &SqlitePool,
    agent_id: &str,
    tenant_id: &str,
    name: &str,
    provider: &str,
    service_name: &str,
    engine_token: &str,
) -> anyhow::Result<AgentRow> {
    let id = agent_id.to_string();
    let created_at = Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO agents (id, tenant_id, name, provider, service_name, engine_token, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(tenant_id)
    .bind(name)
    .bind(provider)
    .bind(service_name)
    .bind(engine_token)
    .bind(&created_at)
    .execute(pool)
    .await?;
    Ok(AgentRow {
        id,
        tenant_id: tenant_id.to_string(),
        name: name.to_string(),
        provider: provider.to_string(),
        service_name: service_name.to_string(),
        engine_token: engine_token.to_string(),
        created_at,
    })
}

pub async fn list_agents(pool: &SqlitePool, tenant_id: &str) -> anyhow::Result<Vec<AgentRow>> {
    Ok(sqlx::query_as::<_, AgentRow>(
        "SELECT id, tenant_id, name, provider, service_name, engine_token, created_at FROM agents WHERE tenant_id = ? ORDER BY created_at",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_agent(
    pool: &SqlitePool,
    tenant_id: &str,
    agent_id: &str,
) -> anyhow::Result<Option<AgentRow>> {
    Ok(sqlx::query_as::<_, AgentRow>(
        "SELECT id, tenant_id, name, provider, service_name, engine_token, created_at FROM agents WHERE tenant_id = ? AND id = ?",
    )
    .bind(tenant_id)
    .bind(agent_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_agent(pool: &SqlitePool, tenant_id: &str, agent_id: &str) -> anyhow::Result<bool> {
    let result = sqlx::query("DELETE FROM agents WHERE tenant_id = ? AND id = ?")
        .bind(tenant_id)
        .bind(agent_id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

fn slugify(input: &str) -> String {
    let mut out = String::new();
    for ch in input.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch);
        } else if ch.is_whitespace() || ch == '-' || ch == '_' {
            if !out.ends_with('-') && !out.is_empty() {
                out.push('-');
            }
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        Uuid::new_v4().simple().to_string()[..8].to_string()
    } else {
        trimmed.to_string()
    }
}

#[allow(dead_code)]
fn parse_ts(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
}
