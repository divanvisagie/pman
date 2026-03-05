use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::io::Read;
use std::path::PathBuf;

use pman::{
    NotesPaths, WcFlags, archive_project, cat_note, create_project, edit_note, generate_skill,
    head_note, init_workspace, less_note, list_projects, read_note, resolve_notes_dir, tail_note,
    update_workspace, verify_workspace, wc_note, write_note,
};

#[derive(Parser)]
#[command(name = "pman", version, about = "Notes project manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new pman workspace
    Init {
        /// Workspace directory (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Update AGENTS.md and canonical skills to latest embedded versions
    Update {
        /// Workspace directory (default: current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Verify workspace setup and report any issues
    Verify {
        /// Workspace directory (default: current directory)
        #[arg(long, default_value = ".")]
        path: PathBuf,
    },
    /// Create a new project note in Notes/Projects
    New {
        /// Project name, or explicit project directory name (<prefix>-<number>-<slug>; default prefix: proj or PMAN_PROJECT_PREFIX)
        name: String,
        /// Status label to record in the registry
        #[arg(long, default_value = "active")]
        status: String,
        /// Area slug to prefix the project directory slug
        #[arg(long)]
        area: Option<String>,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// Archive a project directory into Notes/Archives/Projects
    Archive {
        /// Project directory name or prefix (e.g. proj-0022 or ticket-0022)
        project: String,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// List projects from the registry
    List {
        /// Filter by status (default: active, use 'all' for everything)
        #[arg(long, default_value = "active")]
        status: String,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// Read a note file relative to notes root
    Read {
        /// Note path relative to notes root
        path: PathBuf,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
        /// Inclusive line range (start:end), 1-based
        #[arg(long)]
        lines: Option<String>,
        /// Include line numbers in output
        #[arg(long)]
        numbered: bool,
    },
    /// Replace a note file's full contents
    Write {
        /// Note path relative to notes root
        path: PathBuf,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
        /// Create parent directories if missing
        #[arg(long)]
        create_dirs: bool,
        /// Content to write; if omitted, stdin is used
        #[arg(long)]
        content: Option<String>,
    },
    /// Replace an inclusive line range in a note file
    Edit {
        /// Note path relative to notes root
        path: PathBuf,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
        /// Inclusive line range to replace (start:end), 1-based
        #[arg(long)]
        replace_lines: String,
        /// Replacement text for the selected range
        #[arg(long = "with")]
        with_text: String,
        /// Optional expected text guard for stale-context detection
        #[arg(long)]
        expect: Option<String>,
    },
    /// Notes-scoped cat wrapper
    Cat {
        /// Note path relative to notes root
        path: PathBuf,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// Notes-scoped head wrapper
    Head {
        /// Note path relative to notes root
        path: PathBuf,
        /// Number of lines to show
        #[arg(long, default_value_t = 10)]
        lines: usize,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// Notes-scoped tail wrapper
    Tail {
        /// Note path relative to notes root
        path: PathBuf,
        /// Number of lines to show
        #[arg(long, default_value_t = 10)]
        lines: usize,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// Notes-scoped wc wrapper
    Wc {
        /// Note path relative to notes root
        path: PathBuf,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
        /// Show line count
        #[arg(long)]
        lines: bool,
        /// Show word count
        #[arg(long)]
        words: bool,
        /// Show byte count
        #[arg(long)]
        bytes: bool,
        /// Show character count
        #[arg(long)]
        chars: bool,
    },
    /// Notes-scoped less wrapper
    Less {
        /// Note path relative to notes root
        path: PathBuf,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
    /// Skill operations
    Skill {
        #[command(subcommand)]
        command: SkillCommands,
    },
    /// Start an MCP server for remote note access
    Mcp {
        /// MCP transport to use
        #[arg(long, value_enum, default_value_t = McpTransport::Http)]
        transport: McpTransport,
        /// Port to listen on (HTTP transport only)
        #[arg(long, default_value_t = 3100)]
        port: u16,
        /// Bind address (HTTP transport only)
        #[arg(long, default_value = "0.0.0.0")]
        bind: String,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
        /// TLS certificate file (PEM, HTTP transport only)
        #[arg(long)]
        tls_cert: Option<PathBuf>,
        /// TLS private key file (PEM, HTTP transport only)
        #[arg(long)]
        tls_key: Option<PathBuf>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum McpTransport {
    Http,
    Stdio,
}

#[derive(Subcommand)]
enum SkillCommands {
    /// Print a complete SKILL.md template to stdout
    Generate {
        /// Profile to generate (default: project)
        #[arg(default_value = "project")]
        profile: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { path } => {
            let workspace = if path.is_absolute() {
                path
            } else {
                std::env::current_dir()?.join(path)
            };
            init_workspace(&workspace)?;
        }
        Commands::Update { path } => {
            let workspace = if path.is_absolute() {
                path
            } else {
                std::env::current_dir()?.join(path)
            };
            update_workspace(&workspace)?;
        }
        Commands::Verify { path } => {
            let workspace = if path.is_absolute() {
                path
            } else {
                std::env::current_dir()?.join(path)
            };
            let ok = verify_workspace(&workspace)?;
            if !ok {
                std::process::exit(1);
            }
        }
        Commands::New {
            name,
            status,
            area,
            notes_dir,
        } => {
            let root = resolve_notes_dir(notes_dir)?;
            let paths = NotesPaths::from_root(root);
            let note = create_project(&paths, &name, &status, area.as_deref())?;
            println!("Created {}", note.display());
        }
        Commands::Archive { project, notes_dir } => {
            let root = resolve_notes_dir(notes_dir)?;
            let paths = NotesPaths::from_root(root);
            let dest = archive_project(&paths, &project)?;
            println!("Archived {}", dest.display());
        }
        Commands::List { status, notes_dir } => {
            let root = resolve_notes_dir(notes_dir)?;
            let paths = NotesPaths::from_root(root);
            let filter = if status.eq_ignore_ascii_case("all") {
                None
            } else {
                Some(status.as_str())
            };
            let output = list_projects(&paths, filter)?;
            print!("{output}");
        }
        Commands::Read {
            path,
            notes_dir,
            lines,
            numbered,
        } => {
            let output = read_note(notes_dir, &path, lines.as_deref(), numbered)?;
            print!("{output}");
        }
        Commands::Write {
            path,
            notes_dir,
            create_dirs,
            content,
        } => {
            let body = match content {
                Some(value) => value,
                None => {
                    let mut buffer = String::new();
                    std::io::stdin().read_to_string(&mut buffer)?;
                    buffer
                }
            };
            let target = write_note(notes_dir, &path, &body, create_dirs)?;
            println!("Wrote {}", target.display());
        }
        Commands::Edit {
            path,
            notes_dir,
            replace_lines,
            with_text,
            expect,
        } => {
            let target = edit_note(
                notes_dir,
                &path,
                &replace_lines,
                &with_text,
                expect.as_deref(),
            )?;
            println!("Edited {}", target.display());
        }
        Commands::Cat { path, notes_dir } => {
            let output = cat_note(notes_dir, &path)?;
            print!("{output}");
        }
        Commands::Head {
            path,
            lines,
            notes_dir,
        } => {
            let output = head_note(notes_dir, &path, lines)?;
            print!("{output}");
        }
        Commands::Tail {
            path,
            lines,
            notes_dir,
        } => {
            let output = tail_note(notes_dir, &path, lines)?;
            print!("{output}");
        }
        Commands::Wc {
            path,
            notes_dir,
            lines,
            words,
            bytes,
            chars,
        } => {
            let flags = WcFlags {
                lines,
                words,
                bytes,
                chars,
            };
            let output = wc_note(notes_dir, &path, flags)?;
            print!("{output}");
        }
        Commands::Less { path, notes_dir } => {
            if let Some(output) = less_note(notes_dir, &path)? {
                print!("{output}");
            }
        }
        Commands::Skill { command } => match command {
            SkillCommands::Generate { profile } => {
                let output = generate_skill(&profile)?;
                print!("{output}");
            }
        },
        Commands::Mcp {
            transport,
            port,
            bind,
            notes_dir,
            tls_cert,
            tls_key,
        } => {
            let root = resolve_notes_dir(notes_dir)?;
            let paths = NotesPaths::from_root(root);
            match transport {
                McpTransport::Http => {
                    let tls = match (tls_cert, tls_key) {
                        (Some(cert), Some(key)) => Some((cert, key)),
                        (None, None) => None,
                        _ => {
                            anyhow::bail!("Both --tls-cert and --tls-key must be provided together")
                        }
                    };
                    pman::mcp::serve(paths, &bind, port, tls).await?;
                }
                McpTransport::Stdio => {
                    if tls_cert.is_some() || tls_key.is_some() {
                        anyhow::bail!(
                            "--tls-cert/--tls-key are only supported with --transport http"
                        );
                    }
                    if bind != "0.0.0.0" || port != 3100 {
                        eprintln!("Ignoring --bind/--port because --transport stdio was selected.");
                    }
                    pman::mcp::serve_stdio(paths).await?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mcp_defaults_to_http_transport() {
        let cli = Cli::try_parse_from(["pman", "mcp"]).expect("valid cli");
        match cli.command {
            Commands::Mcp { transport, .. } => assert_eq!(transport, McpTransport::Http),
            _ => panic!("expected mcp command"),
        }
    }

    #[test]
    fn mcp_accepts_stdio_transport() {
        let cli = Cli::try_parse_from(["pman", "mcp", "--transport", "stdio"]).expect("valid cli");
        match cli.command {
            Commands::Mcp { transport, .. } => assert_eq!(transport, McpTransport::Stdio),
            _ => panic!("expected mcp command"),
        }
    }
}
