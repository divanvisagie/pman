use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use axum_server::tls_rustls::RustlsConfig;
use rmcp::ErrorData as McpError;
use rmcp::ServerHandler;
use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
};
use rmcp::service::ServiceExt;
use rmcp::transport::streamable_http_server::StreamableHttpService;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use serde::Deserialize;

use crate::{
    NotesPaths, archive_project, create_project, edit_note, list_projects, read_note, write_note,
};

#[derive(Debug, Clone)]
pub struct PmanMcp {
    paths: Arc<NotesPaths>,
    tool_router: ToolRouter<Self>,
}

// -- Parameter structs --------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NotesReadParams {
    #[schemars(
        description = "Note path relative to the Notes root (e.g. Projects/proj-42-foo/README.md)"
    )]
    path: String,
    #[schemars(description = "Optional inclusive line range, 1-based (e.g. 1:20)")]
    lines: Option<String>,
    #[schemars(description = "Include line numbers in output")]
    numbered: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NotesWriteParams {
    #[schemars(description = "Note path relative to the Notes root")]
    path: String,
    #[schemars(description = "Full content to write to the file")]
    content: String,
    #[schemars(description = "Create parent directories if they don't exist")]
    create_dirs: Option<bool>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct NotesEditParams {
    #[schemars(description = "Note path relative to the Notes root")]
    path: String,
    #[schemars(description = "Inclusive line range to replace, 1-based (e.g. 5:10)")]
    replace_lines: String,
    #[schemars(description = "Replacement text for the selected range")]
    with_text: String,
    #[schemars(
        description = "Optional expected text guard – edit fails if current content of the range doesn't match"
    )]
    expect: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ProjectListParams {
    #[schemars(description = "Filter by status (default: active, use 'all' for everything)")]
    status: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ProjectNewParams {
    #[schemars(description = "Project name (will be slugified for the directory)")]
    name: String,
    #[schemars(description = "Status label (default: draft)")]
    status: Option<String>,
    #[schemars(description = "Area slug to prefix the project directory")]
    area: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
struct ProjectArchiveParams {
    #[schemars(
        description = "Project directory name or prefix (e.g. proj-42 or proj-42-my-project)"
    )]
    project: String,
}

// -- Tool implementations -----------------------------------------------------

#[rmcp::tool_router]
impl PmanMcp {
    pub fn new(paths: NotesPaths) -> Self {
        Self {
            paths: Arc::new(paths),
            tool_router: Self::tool_router(),
        }
    }

    #[rmcp::tool(
        description = "Read a note file from the Notes directory. Returns the file contents."
    )]
    async fn notes_read(
        &self,
        Parameters(params): Parameters<NotesReadParams>,
    ) -> Result<CallToolResult, McpError> {
        let notes_dir = Some(self.paths.root.clone());
        let path = PathBuf::from(&params.path);
        let numbered = params.numbered.unwrap_or(false);
        match read_note(notes_dir, &path, params.lines.as_deref(), numbered) {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[rmcp::tool(description = "Write or replace a note file's full contents.")]
    async fn notes_write(
        &self,
        Parameters(params): Parameters<NotesWriteParams>,
    ) -> Result<CallToolResult, McpError> {
        let notes_dir = Some(self.paths.root.clone());
        let path = PathBuf::from(&params.path);
        let create_dirs = params.create_dirs.unwrap_or(false);
        match write_note(notes_dir, &path, &params.content, create_dirs) {
            Ok(target) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Wrote {}",
                target.display()
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[rmcp::tool(
        description = "Edit a note file by replacing an inclusive line range. Supports an optional expected-text guard to detect stale edits."
    )]
    async fn notes_edit(
        &self,
        Parameters(params): Parameters<NotesEditParams>,
    ) -> Result<CallToolResult, McpError> {
        let notes_dir = Some(self.paths.root.clone());
        let path = PathBuf::from(&params.path);
        match edit_note(
            notes_dir,
            &path,
            &params.replace_lines,
            &params.with_text,
            params.expect.as_deref(),
        ) {
            Ok(target) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Edited {}",
                target.display()
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[rmcp::tool(description = "List projects from the registry. Defaults to active projects.")]
    async fn project_list(
        &self,
        Parameters(params): Parameters<ProjectListParams>,
    ) -> Result<CallToolResult, McpError> {
        let status_str = params.status.unwrap_or_else(|| "active".to_string());
        let filter = if status_str.eq_ignore_ascii_case("all") {
            None
        } else {
            Some(status_str.as_str())
        };
        match list_projects(&self.paths, filter) {
            Ok(output) => Ok(CallToolResult::success(vec![Content::text(output)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[rmcp::tool(description = "Create a new project note in Notes/Projects.")]
    async fn project_new(
        &self,
        Parameters(params): Parameters<ProjectNewParams>,
    ) -> Result<CallToolResult, McpError> {
        let status = params.status.unwrap_or_else(|| "draft".to_string());
        match create_project(&self.paths, &params.name, &status, params.area.as_deref()) {
            Ok(note) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Created {}",
                note.display()
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    #[rmcp::tool(description = "Archive a project by moving it to Notes/Archives/Projects.")]
    async fn project_archive(
        &self,
        Parameters(params): Parameters<ProjectArchiveParams>,
    ) -> Result<CallToolResult, McpError> {
        match archive_project(&self.paths, &params.project) {
            Ok(dest) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Archived {}",
                dest.display()
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }
}

// -- Server handler -----------------------------------------------------------

#[rmcp::tool_handler]
impl ServerHandler for PmanMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "pman".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: Some("pman – Notes Project Manager".to_string()),
                website_url: None,
                icons: None,
            },
            instructions: Some(
                "pman manages a PARA-structured Notes directory. Use the notes_* tools to read, write, and edit note files. Use the project_* tools to create, list, and archive projects."
                    .to_string(),
            ),
        }
    }
}

// -- Server entry point -------------------------------------------------------

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "warn".into()),
        )
        .try_init();
}

pub async fn serve(
    paths: NotesPaths,
    bind: &str,
    port: u16,
    tls: Option<(PathBuf, PathBuf)>,
) -> Result<()> {
    init_tracing();

    let paths_for_closure = paths.clone();
    let service = StreamableHttpService::new(
        move || Ok(PmanMcp::new(paths_for_closure.clone())),
        LocalSessionManager::default().into(),
        Default::default(),
    );

    let router = axum::Router::new().nest_service("/mcp", service);
    let addr: SocketAddr = format!("{bind}:{port}").parse()?;

    if let Some((cert, key)) = tls {
        let config = RustlsConfig::from_pem_file(cert, key).await?;
        tracing::info!("pman MCP server listening on https://{addr}/mcp");
        axum_server::bind_rustls(addr, config)
            .serve(router.into_make_service())
            .await?;
    } else {
        tracing::info!("pman MCP server listening on http://{addr}/mcp");
        axum_server::bind(addr)
            .serve(router.into_make_service())
            .await?;
    }

    Ok(())
}

pub async fn serve_stdio(paths: NotesPaths) -> Result<()> {
    init_tracing();
    tracing::info!("pman MCP server listening on stdio");

    let service = match PmanMcp::new(paths)
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await
    {
        Ok(service) => service,
        Err(err) => {
            let message = err.to_string();
            if message.contains("connection closed")
                && (message.contains("initialize request") || message.contains("initialize"))
            {
                tracing::debug!("stdio connection closed before session establishment: {message}");
                return Ok(());
            }
            return Err(err.into());
        }
    };
    if let Err(err) = service.waiting().await {
        let message = err.to_string();
        // Some MCP clients perform lightweight probes that start the process and
        // close stdio before initialization completes. Treat those disconnects as
        // a clean shutdown instead of surfacing a failure exit code.
        if message.contains("connection closed")
            && (message.contains("initialize request") || message.contains("initialize"))
        {
            tracing::debug!("stdio connection closed before initialization: {message}");
            return Ok(());
        }
        return Err(err.into());
    }

    Ok(())
}
