use axum::extract::{Path, State};
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{any, delete, get, post};
use axum::{Json, Router};
use serde::Serialize;
use serde_json::{json, Value};

use crate::db::{self, CreateAgentRequest, CreateTenantRequest};
use crate::error::{require_admin, ApiError};
use crate::k8s::Provisioner;
use crate::proxy::{forward_http, join_proxy_path};
use crate::state::SharedState;

pub fn router(state: SharedState) -> Router {
    Router::new()
        .route("/v1/health", get(health))
        .route("/v1/tenants", post(create_tenant).get(list_tenants))
        .route("/v1/tenants/{tenant_id}/agents", post(create_agent).get(list_agents))
        .route(
            "/v1/tenants/{tenant_id}/agents/{agent_id}",
            delete(delete_agent),
        )
        .route(
            "/v1/tenants/{tenant_id}/agents/{agent_id}/engine/{*path}",
            any(proxy_engine),
        )
        .with_state(state)
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "houston-control-plane",
    })
}

async fn create_tenant(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Json(body): Json<CreateTenantRequest>,
) -> Result<Json<db::TenantRow>, ApiError> {
    require_admin(&headers, &state.config.admin_token)?;
    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let tenant = db::insert_tenant(&state.db, body.name.trim()).await?;
    state
        .k8s
        .ensure_tenant(&tenant.namespace, &tenant.id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(Json(tenant))
}

async fn list_tenants(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Result<Json<Vec<db::TenantRow>>, ApiError> {
    require_admin(&headers, &state.config.admin_token)?;
    Ok(Json(db::list_tenants(&state.db).await.map_err(ApiError::Internal)?))
}

async fn create_agent(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
    Json(body): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    require_admin(&headers, &state.config.admin_token)?;
    let tenant = db::get_tenant(&state.db, &tenant_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("tenant not found".into()))?;

    if body.name.trim().is_empty() {
        return Err(ApiError::BadRequest("name is required".into()));
    }
    let provider = normalize_provider(&body.provider)?;

    let agent_id = uuid::Uuid::new_v4().simple().to_string();
    let (service_name, engine_token) = state
        .k8s
        .deploy_agent(
            &tenant.namespace,
            &agent_id,
            provider,
            body.api_key.as_deref(),
        )
        .await
        .map_err(ApiError::Internal)?;

    let agent = db::insert_agent(
        &state.db,
        &agent_id,
        &tenant.id,
        body.name.trim(),
        provider,
        &service_name,
        &engine_token,
    )
    .await
    .map_err(ApiError::Internal)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": agent.id,
            "tenant_id": agent.tenant_id,
            "name": agent.name,
            "provider": agent.provider,
            "service_name": agent.service_name,
            "namespace": tenant.namespace,
            "engine_url": Provisioner::engine_base_url(&tenant.namespace, &service_name),
        })),
    ))
}

async fn list_agents(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path(tenant_id): Path<String>,
) -> Result<Json<Vec<Value>>, ApiError> {
    require_admin(&headers, &state.config.admin_token)?;
    let _tenant = db::get_tenant(&state.db, &tenant_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("tenant not found".into()))?;

    let agents = db::list_agents(&state.db, &tenant_id)
        .await
        .map_err(ApiError::Internal)?;
    let tenant = db::get_tenant(&state.db, &tenant_id)
        .await
        .map_err(ApiError::Internal)?
        .unwrap();

    let out = agents
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id,
                "tenant_id": a.tenant_id,
                "name": a.name,
                "provider": a.provider,
                "service_name": a.service_name,
                "namespace": tenant.namespace,
                "engine_url": Provisioner::engine_base_url(&tenant.namespace, &a.service_name),
            })
        })
        .collect();
    Ok(Json(out))
}

async fn delete_agent(
    State(state): State<SharedState>,
    headers: HeaderMap,
    Path((tenant_id, agent_id)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    require_admin(&headers, &state.config.admin_token)?;
    let tenant = db::get_tenant(&state.db, &tenant_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("tenant not found".into()))?;
    let agent = db::get_agent(&state.db, &tenant_id, &agent_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("agent not found".into()))?;

    state
        .k8s
        .delete_agent(&tenant.namespace, &agent.service_name)
        .await
        .map_err(ApiError::Internal)?;
    db::delete_agent(&state.db, &tenant_id, &agent_id)
        .await
        .map_err(ApiError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn proxy_engine(
    State(state): State<SharedState>,
    headers: HeaderMap,
    method: Method,
    Path((tenant_id, agent_id, path)): Path<(String, String, String)>,
    uri: axum::http::Uri,
    body: axum::body::Body,
) -> Result<impl IntoResponse, ApiError> {
    require_admin(&headers, &state.config.admin_token)?;
    let tenant = db::get_tenant(&state.db, &tenant_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("tenant not found".into()))?;
    let agent = db::get_agent(&state.db, &tenant_id, &agent_id)
        .await
        .map_err(ApiError::Internal)?
        .ok_or_else(|| ApiError::NotFound("agent not found".into()))?;

    let base = Provisioner::engine_base_url(&tenant.namespace, &agent.service_name);
    let tail = join_proxy_path(&path);
    let path_and_query = if let Some(q) = uri.query() {
        format!("{tail}?{q}")
    } else {
        tail
    };

    forward_http(
        &state.http,
        &base,
        &agent.engine_token,
        method,
        &path_and_query,
        headers,
        body,
    )
    .await
}

fn normalize_provider(raw: &str) -> Result<&'static str, ApiError> {
    match raw.to_lowercase().as_str() {
        "anthropic" | "claude" => Ok("anthropic"),
        "openai" | "codex" | "gpt" => Ok("openai"),
        "gemini" | "google" => Ok("gemini"),
        other => Err(ApiError::BadRequest(format!(
            "unsupported provider '{other}', use anthropic|openai|gemini"
        ))),
    }
}
