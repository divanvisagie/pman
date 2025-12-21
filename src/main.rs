use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

use pman::{archive_project, create_project, resolve_notes_dir, NotesPaths};

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
        Commands::New {
            name,
            status,
            notes_dir,
        } => {
            let root = resolve_notes_dir(notes_dir)?;
            let paths = NotesPaths::from_root(root);
            let note = create_project(&paths, &name, &status)?;
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
