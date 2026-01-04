use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use pman::{archive_project, create_project, init_workspace, resolve_notes_dir, update_workspace, verify_workspace, NotesPaths};

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
    /// Update CLAUDE.md and skills to latest embedded versions
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
        /// Project name (used for title and slug)
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
        /// Project directory name or prefix (e.g. proj-0022)
        project: String,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
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
        Commands::Archive {
            project,
            notes_dir,
        } => {
            let root = resolve_notes_dir(notes_dir)?;
            let paths = NotesPaths::from_root(root);
            let dest = archive_project(&paths, &project)?;
            println!("Archived {}", dest.display());
        }
    }

    Ok(())
}
