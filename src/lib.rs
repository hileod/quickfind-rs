pub mod cli;
pub mod index;
pub mod scanner;
pub mod search;
pub mod search_engine;
pub mod storage;
pub mod turso_storage;
pub mod windows_indexer;

use std::error::Error;
use std::time::Instant;

use cli::{Cli, Command};
use scanner::build_index;
use search_engine::search;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn run() -> Result<()> {
    match cli::parse()? {
        Cli {
            command:
                Command::Index {
                    roots,
                    output,
                    threads,
                },
        } => {
            let started = Instant::now();
            let entries = build_index(roots, threads)?;
            storage::write_index(&output, &entries)?;
            let metadata = turso_storage::metadata_path_for(&output);
            turso_storage::write_metadata(&metadata, &entries)?;
            println!(
                "Indexed {} files in {:.2}s -> {}",
                entries.len(),
                started.elapsed().as_secs_f64(),
                output.display()
            );
            println!("Metadata -> {}", metadata.display());
        }
        Cli {
            command:
                Command::Search {
                    query,
                    index,
                    limit,
                },
        } => {
            let started = Instant::now();
            let entries = storage::read_index(&index)?;
            let loaded_at = started.elapsed();
            let searched = Instant::now();
            let matches = search(&entries, &query, limit);

            for item in matches {
                println!("{}", item.entry.path);
            }

            eprintln!(
                "Loaded {} entries in {:.3}s, searched in {:.3}s",
                entries.len(),
                loaded_at.as_secs_f64(),
                searched.elapsed().as_secs_f64()
            );
        }
        Cli {
            command: Command::Stats { index },
        } => {
            let started = Instant::now();
            let entries = storage::read_index(&index)?;
            let bytes: usize = entries.iter().map(|entry| entry.path.len()).sum();
            let metadata = turso_storage::metadata_path_for(&index);
            println!("Index: {}", index.display());
            if metadata.exists() {
                println!("Metadata: {}", metadata.display());
            }
            println!("Files: {}", entries.len());
            println!("Path bytes: {}", bytes);
            println!("Loaded in: {:.3}s", started.elapsed().as_secs_f64());
        }
        Cli {
            command: Command::Help,
        } => cli::print_help(),
    }

    Ok(())
}
