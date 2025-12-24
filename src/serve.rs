use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, Redirect, Response, IntoResponse},
    routing::get,
    Router,
};
use comrak::{markdown_to_html, Options};
use regex::Regex;
use std::path::PathBuf;
use std::sync::Arc;

const MAIN_JS: &str = include_str!("../assets/main.js");

struct AppState {
    notes_dir: PathBuf,
}

pub async fn run_server(notes_dir: PathBuf, port: u16) -> Result<()> {
    let state = Arc::new(AppState { notes_dir });

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/{*path}", get(handle_path))
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    println!("Serving notes at http://localhost:{port}");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_root(State(state): State<Arc<AppState>>) -> Result<Html<String>, StatusCode> {
    serve_path(&state.notes_dir, &state.notes_dir).await
}

async fn handle_path(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
) -> Response {
    let full_path = state.notes_dir.join(&path);

    // Redirect directories without trailing slash
    if full_path.is_dir() && !path.ends_with('/') {
        return Redirect::permanent(&format!("/{path}/")).into_response();
    }

    match serve_path(&state.notes_dir, &full_path).await {
        Ok(html) => html.into_response(),
        Err(status) => status.into_response(),
    }
}

async fn serve_path(notes_dir: &PathBuf, path: &PathBuf) -> Result<Html<String>, StatusCode> {
    // Security: ensure path is within notes_dir
    let canonical = path.canonicalize().map_err(|_| StatusCode::NOT_FOUND)?;
    let notes_canonical = notes_dir.canonicalize().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !canonical.starts_with(&notes_canonical) {
        return Err(StatusCode::FORBIDDEN);
    }

    let file_tree = render_file_tree(&notes_canonical, &notes_canonical)?;

    if canonical.is_file() {
        if canonical.extension().is_some_and(|ext| ext == "md") {
            let content = std::fs::read_to_string(&canonical).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let html = render_markdown(&content);
            let title = canonical
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Note");
            Ok(Html(wrap_html(title, &html, &file_tree)))
        } else {
            Err(StatusCode::NOT_FOUND)
        }
    } else if canonical.is_dir() {
        // Check for README.md or INDEX.md
        let readme = canonical.join("README.md");
        let index = canonical.join("INDEX.md");

        if readme.exists() {
            let content = std::fs::read_to_string(&readme).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let html = render_markdown(&content);
            Ok(Html(wrap_html("Notes", &html, &file_tree)))
        } else if index.exists() {
            let content = std::fs::read_to_string(&index).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let html = render_markdown(&content);
            Ok(Html(wrap_html("Notes", &html, &file_tree)))
        } else {
            let html = render_directory(&canonical, notes_dir)?;
            let dir_name = canonical
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Notes");
            Ok(Html(wrap_html(dir_name, &html, &file_tree)))
        }
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

fn process_wiki_links(content: &str) -> String {
    let re = Regex::new(r"\[\[([^\]|]+)(?:\|([^\]]+))?\]\]").expect("valid regex");

    re.replace_all(content, |caps: &regex::Captures| {
        let target = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let display = caps.get(2).map(|m| m.as_str()).unwrap_or(target);

        // Build the link path
        let path = if target.ends_with(".md") {
            format!("/{}", target)
        } else {
            format!("/{}.md", target)
        };

        format!("[{}]({})", display, path)
    }).to_string()
}

fn render_markdown(content: &str) -> String {
    let processed = process_wiki_links(content);

    let mut options = Options::default();
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.render.unsafe_ = true;

    markdown_to_html(&processed, &options)
}

fn render_file_tree(dir: &PathBuf, notes_root: &PathBuf) -> Result<String, StatusCode> {
    fn render_tree_recursive(dir: &PathBuf, notes_root: &PathBuf, depth: usize) -> Result<String, StatusCode> {
        if depth > 3 {
            return Ok(String::new());
        }

        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .filter_map(|e| e.ok())
            .collect();

        entries.sort_by_key(|e| e.file_name());

        let mut html = String::from("<ul>\n");

        for entry in entries {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            if name_str.starts_with('.') || name_str.starts_with('_') {
                continue;
            }

            let file_type = entry.file_type().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            let entry_path = entry.path();
            let relative_path = entry_path
                .strip_prefix(notes_root)
                .unwrap_or(&entry_path)
                .to_string_lossy();

            if file_type.is_dir() {
                let children = render_tree_recursive(&entry_path, notes_root, depth + 1)?;
                if children.contains("<li>") || depth < 1 {
                    html.push_str(&format!(
                        "<li class=\"dir\"><span class=\"toggle\"></span><a href=\"/{path}/\">{name}</a>{children}</li>\n",
                        path = relative_path,
                        name = name_str,
                        children = children
                    ));
                }
            } else if name_str.ends_with(".md") && depth > 0 {
                html.push_str(&format!(
                    "<li><a href=\"/{path}\">{name}</a></li>\n",
                    path = relative_path,
                    name = name_str
                ));
            }
        }

        html.push_str("</ul>");
        Ok(html)
    }

    let mut html = String::from("<nav class=\"file-tree\"><a href=\"/\">Notes</a>");
    html.push_str(&render_tree_recursive(dir, notes_root, 0)?);
    html.push_str("</nav>");
    Ok(html)
}

fn render_directory(dir: &PathBuf, notes_dir: &PathBuf) -> Result<String, StatusCode> {
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.file_name());

    let mut html = String::from("<ul class=\"file-listing\">\n");

    // Add parent link if not at root
    if dir != notes_dir {
        html.push_str("  <li><a href=\"..\">..</a></li>\n");
    }

    for entry in entries {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        // Skip hidden files
        if name_str.starts_with('.') || name_str.starts_with('_') {
            continue;
        }

        let file_type = entry.file_type().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        if file_type.is_dir() {
            html.push_str(&format!(
                "  <li><a href=\"{name}/\">{name}/</a></li>\n",
                name = name_str
            ));
        } else if name_str.ends_with(".md") {
            html.push_str(&format!(
                "  <li><a href=\"{name}\">{name}</a></li>\n",
                name = name_str
            ));
        }
    }

    html.push_str("</ul>");
    Ok(html)
}

fn wrap_html(title: &str, content: &str, file_tree: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - pman</title>
    <link rel="stylesheet" href="https://divanv.com/css/main.css">
    <style>
        body {{
            display: flex;
            margin: 0;
            min-height: 100vh;
            position: relative;
        }}
        .sidebar {{
            display: flex;
            position: sticky;
            top: 0;
            height: 100vh;
        }}
        .file-tree {{
            width: 280px;
            min-width: 150px;
            max-width: 600px;
            padding: 1rem 1.5rem;
            overflow-y: auto;
            height: 100vh;
            background: var(--background-color);
        }}
        .resize-handle {{
            width: 4px;
            cursor: col-resize;
            background: var(--subtitle-color);
            transition: background 0.2s;
        }}
        .resize-handle:hover,
        .resize-handle.dragging {{
            background: var(--accent-color);
        }}
        .file-tree ul {{
            list-style: none;
            padding-left: 1rem;
            margin: 0;
        }}
        .file-tree > ul {{
            padding-left: 0;
        }}
        .file-tree li {{
            padding: 0.15rem 0;
        }}
        .file-tree li.dir {{
            position: relative;
        }}
        .file-tree .toggle {{
            display: inline-block;
            width: 1rem;
            cursor: pointer;
            user-select: none;
        }}
        .file-tree .toggle::before {{
            content: '▼';
            font-size: 0.6rem;
            color: var(--subtitle-color);
            transition: transform 0.15s;
            display: inline-block;
        }}
        .file-tree li.dir.collapsed .toggle::before {{
            transform: rotate(-90deg);
        }}
        .file-tree li.dir.collapsed > ul {{
            display: none;
        }}
        .file-tree a {{
            text-decoration: none;
            color: var(--text-color);
        }}
        .file-tree a:hover {{
            color: var(--accent-color);
        }}
        .file-tree > a {{
            font-weight: bold;
            font-size: 1.1rem;
            display: inline-block;
            margin-bottom: 0.75rem;
            border-bottom: 2px solid var(--accent-color);
            padding-bottom: 2px;
        }}
        main {{
            flex: 1;
            padding: 2rem 2.5rem;
            max-width: 800px;
            overflow-y: auto;
        }}
        main h1:first-child {{
            margin-top: 0;
        }}
        .file-listing {{
            list-style: none;
            padding: 0;
        }}
        .file-listing li {{
            padding: 0.5rem 0;
            border-bottom: 1px solid var(--subtitle-color);
        }}
        .file-listing a {{
            text-decoration: none;
        }}
        .file-listing a:hover {{
            color: var(--accent-color);
        }}
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 1rem 0;
        }}
        th, td {{
            border: 1px solid var(--subtitle-color);
            padding: 0.5rem 0.75rem;
            text-align: left;
        }}
        th {{
            background: var(--code-background);
        }}
        code {{
            background: var(--code-background);
            padding: 0.1rem 0.3rem;
            border: 1px solid var(--subtitle-color);
        }}
        pre {{
            background: var(--code-background);
            padding: 1rem;
            border: 1px solid var(--subtitle-color);
            overflow-x: auto;
        }}
        pre code {{
            border: none;
            padding: 0;
        }}
    </style>
</head>
<body>
    <div class="sidebar">
        {file_tree}
        <div class="resize-handle"></div>
    </div>
    <main>
        {content}
    </main>
    <script>{js}</script>
</body>
</html>"#,
        title = title,
        content = content,
        file_tree = file_tree,
        js = MAIN_JS
    )
}
