//! Embedded MCP (Model Context Protocol) server.
//!
//! Exposes a lighting-focused tool set over a localhost Streamable HTTP
//! endpoint so external agents can drive the lamp. The endpoint is bound to
//! `127.0.0.1` and, when a token is configured, requires a bearer token.
//!
//! Tools never mutate `Config` directly: they call the shared helpers in
//! `commands.rs`, which run the same `update_config` path as the Tauri
//! commands (revision bump -> worker wake-up -> persistence).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Request, State as AxumState},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    Router,
};
use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock},
    tool, tool_handler, tool_router,
    transport::streamable_http_server::{
        session::local::LocalSessionManager, StreamableHttpServerConfig, StreamableHttpService,
    },
    ErrorData as McpError, ServerHandler,
};
use schemars::JsonSchema;
use serde::Deserialize;
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use crate::app_state::AppState;
use crate::commands::{
    apply_lamp_mode, apply_lamp_power, apply_static_scene, apply_sync_enabled,
};
use crate::config::{LampMode, StaticColor};

#[derive(Debug, Deserialize, JsonSchema)]
struct SetColorArgs {
    /// Hex color, e.g. `#ff8800` or `#f80`.
    hex: String,
    /// Brightness `10..=1000`. Defaults to the current brightness.
    #[serde(default)]
    brightness: Option<u16>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetBrightnessArgs {
    /// Brightness `10..=1000`.
    brightness: u16,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetModeArgs {
    /// `sync` follows the sampled screen, `static` shows the fixed color.
    mode: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetPowerArgs {
    /// `true` turns the lamp on, `false` turns it off.
    on: bool,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SetSyncArgs {
    /// `true` resumes following the screen, `false` pauses sampling.
    enabled: bool,
}

fn text_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![ContentBlock::text(value.to_string())])
}

fn error_result(message: String) -> CallToolResult {
    CallToolResult::error(vec![ContentBlock::text(message)])
}

/// MCP handler. A fresh instance is created per request/session by the
/// service factory; `AppHandle` is a cheap, thread-safe clone.
#[derive(Clone)]
struct LampMcpServer {
    app: AppHandle,
}

#[tool_router]
impl LampMcpServer {
    fn new(app: AppHandle) -> Self {
        Self { app }
    }

    #[tool(
        description = "Get the current lamp state: power, mode, screen-sync flag, static color and whether a lamp is configured."
    )]
    fn get_lamp_state(&self) -> Result<CallToolResult, McpError> {
        let state = self.app.state::<Arc<AppState>>();
        let config = state.config.read().unwrap();

        let configured = config.lamp_connection.lamp_id.is_some()
            && config.lamp_connection.lamp_ip.is_some()
            && config.lamp_connection.lamp_key.is_some();

        Ok(text_result(serde_json::json!({
            "powered": config.powered,
            "mode": config.lamp_mode,
            "sync_enabled": config.sync_enabled,
            "static_color": config.static_color,
            "lamp_configured": configured,
        })))
    }

    #[tool(
        description = "Show a fixed color on the lamp. Switches to static mode and turns the lamp on. `brightness` is 10..=1000 and defaults to the current brightness."
    )]
    fn set_color(
        &self,
        Parameters(args): Parameters<SetColorArgs>,
    ) -> Result<CallToolResult, McpError> {
        let state = self.app.state::<Arc<AppState>>();

        let brightness = match args.brightness {
            Some(brightness) => brightness,
            None => state.config.read().unwrap().static_color.brightness,
        }
        .clamp(10, 1000);

        let color = StaticColor {
            hex: args.hex,
            brightness,
        };

        match apply_static_scene(&self.app, state.inner(), color.clone(), true) {
            Ok(()) => Ok(text_result(serde_json::json!({
                "ok": true,
                "mode": "static",
                "powered": true,
                "static_color": color,
            }))),
            Err(error) => Ok(error_result(error)),
        }
    }

    #[tool(
        description = "Set the brightness of the static color (10..=1000) and switch to static mode. Does not change power."
    )]
    fn set_brightness(
        &self,
        Parameters(args): Parameters<SetBrightnessArgs>,
    ) -> Result<CallToolResult, McpError> {
        let state = self.app.state::<Arc<AppState>>();

        let (hex, powered) = {
            let config = state.config.read().unwrap();
            (config.static_color.hex.clone(), config.powered)
        };

        let color = StaticColor {
            hex,
            brightness: args.brightness.clamp(10, 1000),
        };

        match apply_static_scene(&self.app, state.inner(), color.clone(), powered) {
            Ok(()) => Ok(text_result(serde_json::json!({
                "ok": true,
                "mode": "static",
                "static_color": color,
            }))),
            Err(error) => Ok(error_result(error)),
        }
    }

    #[tool(
        description = "Set the lamp mode: 'sync' follows the sampled screen, 'static' shows the fixed color."
    )]
    fn set_mode(
        &self,
        Parameters(args): Parameters<SetModeArgs>,
    ) -> Result<CallToolResult, McpError> {
        let mode = match args.mode.to_lowercase().as_str() {
            "sync" => LampMode::Sync,
            "static" | "color" | "colour" => LampMode::Static,
            other => {
                return Err(McpError::invalid_params(
                    format!("unknown mode '{other}' (expected 'sync' or 'static')"),
                    None,
                ))
            }
        };

        let state = self.app.state::<Arc<AppState>>();
        match apply_lamp_mode(&self.app, state.inner(), mode) {
            Ok(()) => Ok(text_result(serde_json::json!({ "ok": true, "mode": mode }))),
            Err(error) => Ok(error_result(error)),
        }
    }

    #[tool(description = "Turn the lamp on or off.")]
    fn set_power(
        &self,
        Parameters(args): Parameters<SetPowerArgs>,
    ) -> Result<CallToolResult, McpError> {
        let state = self.app.state::<Arc<AppState>>();
        match apply_lamp_power(&self.app, state.inner(), args.on) {
            Ok(()) => Ok(text_result(
                serde_json::json!({ "ok": true, "powered": args.on }),
            )),
            Err(error) => Ok(error_result(error)),
        }
    }

    #[tool(description = "Enable or pause following the screen (screen sampling).")]
    fn set_sync_enabled(
        &self,
        Parameters(args): Parameters<SetSyncArgs>,
    ) -> Result<CallToolResult, McpError> {
        let state = self.app.state::<Arc<AppState>>();
        match apply_sync_enabled(&self.app, state.inner(), args.enabled) {
            Ok(()) => Ok(text_result(
                serde_json::json!({ "ok": true, "sync_enabled": args.enabled }),
            )),
            Err(error) => Ok(error_result(error)),
        }
    }
}

#[tool_handler(
    name = "lampz-sync",
    instructions = "Controls the LampZ Sync lamp. Use set_color to show a color, set_power to turn it on/off, set_mode to choose between following the screen ('sync') and a fixed color ('static')."
)]
impl ServerHandler for LampMcpServer {}

/// Bearer-token check. The token is read live from config so rotating it needs
/// no restart; an empty/`None` token leaves the endpoint open.
async fn auth_middleware(
    AxumState(app): AxumState<AppHandle>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let required = {
        let state = app.state::<Arc<AppState>>();
        let config = state.config.read().unwrap();
        config
            .mcp
            .token
            .clone()
            .filter(|token| !token.is_empty())
    };

    let Some(token) = required else {
        return Ok(next.run(request).await);
    };

    let provided = headers
        .get("authorization")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));

    if provided == Some(token.as_str()) {
        Ok(next.run(request).await)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

fn build_router(app: &AppHandle, cancel: CancellationToken) -> Router {
    let service = StreamableHttpService::new(
        {
            let app = app.clone();
            move || Ok(LampMcpServer::new(app.clone()))
        },
        LocalSessionManager::default().into(),
        StreamableHttpServerConfig::default().with_cancellation_token(cancel),
    );

    Router::new()
        .nest_service("/mcp", service)
        .layer(middleware::from_fn_with_state(app.clone(), auth_middleware))
}

/// Watches `Config::mcp` and (re)binds the server whenever it is enabled or the
/// port changes. Runs on its own thread + tokio runtime so it never competes
/// with Tauri's runtime.
pub fn spawn_mcp_supervisor(app: AppHandle) {
    std::thread::spawn(move || {
        let runtime = match tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
        {
            Ok(runtime) => runtime,
            Err(error) => {
                eprintln!("Failed to start MCP runtime: {error}");
                return;
            }
        };

        let mut active: Option<(u16, CancellationToken)> = None;

        loop {
            let (enabled, port) = {
                let state = app.state::<Arc<AppState>>();
                let config = state.config.read().unwrap();
                (config.mcp.enabled, config.mcp.port)
            };

            if let Some((current_port, _)) = active.as_ref() {
                if !enabled || *current_port != port {
                    if let Some((_, token)) = active.take() {
                        token.cancel();
                    }
                }
            }

            if enabled && active.is_none() {
                let addr = SocketAddr::from(([127, 0, 0, 1], port));

                match runtime.block_on(tokio::net::TcpListener::bind(addr)) {
                    Ok(listener) => {
                        let cancel = CancellationToken::new();
                        let router = build_router(&app, cancel.child_token());
                        let shutdown = cancel.clone();

                        runtime.spawn(async move {
                            let _ = axum::serve(listener, router)
                                .with_graceful_shutdown(async move {
                                    shutdown.cancelled().await;
                                })
                                .await;
                        });

                        active = Some((port, cancel));
                        eprintln!("MCP server listening on http://{addr}/mcp");
                    }
                    Err(error) => {
                        eprintln!("MCP server failed to bind {addr}: {error}");
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
