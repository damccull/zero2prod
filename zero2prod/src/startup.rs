use std::{net::TcpListener, sync::Arc};

use axum::{
    extract::FromRef,
    middleware,
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use axum_flash::Key;
use axum_session::{SessionConfig, SessionLayer, SessionRedisPool, SessionStore};
use hyper::server::conn::AddrIncoming;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    authentication::reject_anonymous_users,
    configuration::{DatabaseSettings, Settings},
    routes::{
        admin_dashboard, change_password, change_password_form, confirm, home, log_out, login,
        login_form,
        newsletters::{newsletters_publish_form, publish_newsletter},
    },
    telemetry::RouterExt,
};
use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub type AppServer = Server<AddrIncoming, IntoMakeService<Router>>;

pub struct Application {
    port: u16,
    server: AppServer,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, anyhow::Error> {
        // Get database pool
        let db_pool = get_db_pool(&configuration.database);

        // Build a redis connection
        let redis = redis::Client::open(configuration.redis.uri.expose_secret().as_str())?;
        // Create a session store
        let session_config = SessionConfig::new();
        let session_store =
            SessionStore::<SessionRedisPool>::new(Some(redis.into()), session_config);

        // Build an email client
        let email_client = configuration.email_client.client();

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(address.to_string()).map_err(|e| {
            tracing::error!("failed to bind port {}", address);
            e
        })?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            db_pool,
            email_client,
            configuration.application.base_url,
            configuration.application.hmac_secret,
            session_store,
        );
        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> hyper::Result<()> {
        self.server.await
    }
}

/// Get a database connection pool.
pub fn get_db_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
    hmac_secret: Secret<String>,
    session_store: SessionStore<SessionRedisPool>,
) -> AppServer {
    // Build app state
    let app_state = AppState {
        db_pool,
        email_client: Arc::new(email_client),
        base_url: ApplicationBaseUrl(base_url),
        flash_config: axum_flash::Config::new(Key::from(hmac_secret.expose_secret().as_bytes())),
    };

    // Routes that need to not have a session applied
    let router_no_session = Router::new().route("/health_check", get(health_check));

    // All admin section routes
    let router_for_admin_section = Router::new()
        .route("/admin/dashboard", get(admin_dashboard))
        .route("/admin/newsletters", get(newsletters_publish_form))
        .route("/admin/newsletters", post(publish_newsletter))
        .route("/admin/password", get(change_password_form))
        .route("/admin/password", post(change_password))
        .route("/admin/logout", post(log_out))
        .layer(middleware::from_fn(reject_anonymous_users));

    // All routes that should be a care about session
    let router_with_session = Router::new()
        .route("/", get(home))
        .route("/login", get(login_form))
        .route("/login", post(login))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .merge(router_for_admin_section)
        .layer(SessionLayer::new(session_store));

    // Create a router that will contain and match all routes for the application
    let app = Router::new()
        .merge(router_no_session)
        .merge(router_with_session)
        .add_axum_tracing_layer()
        .with_state(app_state);

    // Start the axum server and set up to use supplied listener
    axum::Server::from_tcp(listener)
        .expect("failed to create server from listener")
        .serve(app.into_make_service())
}

#[derive(Clone)]
pub struct AppState {
    db_pool: PgPool,
    email_client: Arc<EmailClient>,
    base_url: ApplicationBaseUrl,
    flash_config: axum_flash::Config,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}

impl FromRef<AppState> for Arc<EmailClient> {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.email_client.clone()
    }
}

impl FromRef<AppState> for ApplicationBaseUrl {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.base_url.clone()
    }
}

impl FromRef<AppState> for axum_flash::Config {
    fn from_ref(app_state: &AppState) -> axum_flash::Config {
        app_state.flash_config.clone()
    }
}

#[derive(Clone)]
pub struct ApplicationBaseUrl(pub String);

#[derive(Clone)]
pub struct HmacSecret(pub Secret<String>);
