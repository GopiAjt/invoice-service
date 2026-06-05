use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::state::AppState;

pub async fn api_key_auth(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());

    let Some(api_key) = api_key else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let business = sqlx::query_as::<_, (Uuid,)>(
        r#"
        SELECT id
        FROM businesses
        WHERE api_key_hash = $1
        LIMIT 1
        "#,
    )
    .bind(api_key)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let Some((business_id,)) = business else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // Store business_id for handlers
    request.extensions_mut().insert(business_id);

    Ok(next.run(request).await)
}