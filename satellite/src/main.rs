use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Arc, RwLock},
    time::Duration,
    str::from_utf8,
    fmt::Display,
    time::SystemTime,
};
use axum::{
    async_trait,
    body::Bytes,
    error_handling::HandleErrorLayer,
    extract::{RequestParts, Path, Extension, FromRequest, TypedHeader},
    headers::{authorization::{Bearer, Basic}, Authorization},
    handler::Handler,
    http::{HeaderValue, Method, StatusCode, header::WWW_AUTHENTICATE, HeaderMap},
    response::{IntoResponse, Response},
    routing::get,
    Router, Json,
};
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    compression::CompressionLayer,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower::{BoxError, ServiceBuilder};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Algorithm, Validation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Default)]
struct AppState {
    db: HashMap<String, Bytes>,
}

type SharedState = Arc<RwLock<AppState>>;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

impl Display for Claims {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Email: {}\nCompany: {}", self.sub, self.aud)
    }
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequest<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request(req: &mut RequestParts<S>) -> Result<Self, Self::Rejection> {
        let bearer_wrapper =
            TypedHeader::<Authorization<Bearer>>::from_request(req)
                .await
                .map_err(|_| AuthError::InvalidToken);
        if let Ok(TypedHeader(Authorization(bearer))) = bearer_wrapper {
            let token_data = decode::<Claims>(bearer.token(), &KEYS.decoding, &Validation::default())
                .map_err(|_| AuthError::InvalidToken)?;

            Ok(token_data.claims)
        } else {
            let TypedHeader(Authorization(basic)) =
                TypedHeader::<Authorization<Basic>>::from_request(req)
                    .await
                    .map_err(|_| AuthError::InvalidToken)?;
            if basic.username() != "vimarrow" && basic.password() != "toor21" {
                return Err(AuthError::WrongCredentials);
            }
            let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let claims = Claims {
                sub: basic.username().to_owned(),
                aud: "orbit1".to_owned(),
                iss: "satellite".to_owned(),
                iat: now,
                exp: now + 7200,
                nbf: now - 60,
            };
            Ok(claims)
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let mut headers = HeaderMap::new();
        headers.insert(WWW_AUTHENTICATE, HeaderValue::from_str("Basic realm='orbit1'").unwrap());
        (headers, StatusCode::UNAUTHORIZED).into_response()
    }
}

struct Keys {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    aud: String,
    iss: String,
    exp: u64,
    nbf: u64,
    iat: u64,
}

#[derive(Debug, Serialize)]
struct AuthBody {
    access_token: String,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct AuthPayload {
    client_id: String,
    client_secret: String,
}

#[derive(Debug)]
enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

#[tokio::main]
async fn main() {

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "key_value_store=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors_layer = CorsLayer::new()
        .allow_origin("http://localhost:1234".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::DELETE]);

    let app = Router::new()
        .route("/session", get(session))
        .route("/kvlist", get(kv_list_keys))
        .route(
            "/kv/:key",
            get(kv_get.layer(CompressionLayer::new()))
            .post(kv_set)
            .delete(kv_delete),
        )
        .layer(cors_layer)
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .layer(Extension(SharedState::default()))
                .into_inner(),        )
        .fallback(handler_404.into_service());

    serve(app).await;

}

async fn serve(app: Router) {
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn kv_list_keys(Extension(state): Extension<SharedState>) -> Json<String> {
    let db = &state.read().unwrap().db;

    let result = db.keys()
        .map(|key| key.to_string())
        .collect::<Vec<String>>();

    Json(serde_json::to_string::<Vec<String>>(&result).unwrap())
}

async fn kv_set(Path(key): Path<String>, Extension(state): Extension<SharedState>, bytes: Bytes) -> impl IntoResponse {
    state.write().unwrap().db.insert(key, bytes);

    return (
        StatusCode::NO_CONTENT,
        "Ok",
    );
}

async fn kv_delete(Path(key): Path<String>, Extension(state): Extension<SharedState>) -> impl IntoResponse {
    state.write().unwrap().db.remove(&key);

    return (
        StatusCode::NO_CONTENT,
        "Ok",
    );
}

async fn kv_get(
    Path(key): Path<String>,
    Extension(state): Extension<SharedState>,
) -> impl IntoResponse {
    let db = &state.read().unwrap().db;

    if let Some(value) = db.get(&key) {
        let content = from_utf8(&value.clone()).unwrap().to_owned();
        return (
            StatusCode::OK,
            Json(content),
        );
    }
    return (
        StatusCode::NOT_FOUND,
        Json("Nothing to see here.".to_owned()),
    );
}

async fn session(claims: Claims) -> Result<Json<AuthBody>, AuthError> {
    let jwt_head = jsonwebtoken::Header {
        typ: Some("JWT".to_string()),
        alg: Algorithm::HS256,
        cty: None,
        jku: None,
        jwk: None,
        kid: None,
        x5u: None,
        x5c: None,
        x5t: None,
        x5t_s256: None,
    };
    let token = encode(&jwt_head, &claims, &KEYS.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(Json(AuthBody::new(token)))
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Cow::from("Service is overloaded, try again later"),
        );
    }

    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Cow::from(format!("Unhandled internal error: {}", error)),
    )
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Nothing to see here.")
}

