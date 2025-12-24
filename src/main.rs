use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use pman::{archive_project, create_project, resolve_notes_dir, serve, NotesPaths};

#[derive(Parser)]
#[command(name = "pman", version, about = "Notes project manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
    /// Serve Notes directory as a web interface
    Serve {
        /// Port to listen on
        #[arg(long, default_value = "8989")]
        port: u16,
        /// Override Notes root directory
        #[arg(long)]
        notes_dir: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
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
        Commands::Serve { port, notes_dir } => {
            let root = resolve_notes_dir(notes_dir)?;
            serve::run_server(root, port).await?;
        }
    }

    Ok(())
}
