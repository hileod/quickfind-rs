use std::env;
use std::path::PathBuf;
use std::thread;

use crate::Result;

#[derive(Debug)]
pub struct Cli {
    pub command: Command,
}

#[derive(Debug)]
pub enum Command {
    Index {
        roots: Vec<PathBuf>,
        output: PathBuf,
        threads: usize,
    },
    Search {
        query: String,
        index: PathBuf,
        limit: usize,
    },
    Stats {
        index: PathBuf,
    },
    Help,
}

pub fn parse() -> Result<Cli> {
    parse_from(env::args().skip(1))
}

fn parse_from(mut args: impl Iterator<Item = String>) -> Result<Cli> {
    let command = match args.next().as_deref() {
        Some("index") | Some("rebuild") => parse_index(args)?,
        Some("search") | Some("find") => parse_search(args)?,
        Some("stats") => parse_stats(args)?,
        Some("-h") | Some("--help") | None => Command::Help,
        Some(value) => return Err(format!("unknown command: {value}").into()),
    };

    Ok(Cli { command })
}

fn parse_index(mut args: impl Iterator<Item = String>) -> Result<Command> {
    let mut roots = Vec::new();
    let mut output = default_index_path();
    let mut threads = default_thread_count();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" => output = expect_path_arg(&mut args, &arg)?,
            "-j" | "--threads" => {
                threads = parse_thread_count(&expect_string_arg(&mut args, &arg)?)?;
            }
            "-h" | "--help" => return Ok(Command::Help),
            value => roots.push(PathBuf::from(value)),
        }
    }

    if roots.is_empty() {
        roots.push(default_root());
    }

    Ok(Command::Index {
        roots,
        output,
        threads,
    })
}

fn parse_search(mut args: impl Iterator<Item = String>) -> Result<Command> {
    let mut index = default_index_path();
    let mut limit = 50;
    let mut query_parts = Vec::new();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--index" => index = expect_path_arg(&mut args, &arg)?,
            "-n" | "--limit" => {
                limit = expect_string_arg(&mut args, &arg)?.parse::<usize>()?.max(1)
            }
            "-h" | "--help" => return Ok(Command::Help),
            value => query_parts.push(value.to_string()),
        }
    }

    if query_parts.is_empty() {
        return Err("missing search query".into());
    }

    let query = query_parts.join(" ");
    Ok(Command::Search {
        query,
        index,
        limit,
    })
}

fn parse_stats(mut args: impl Iterator<Item = String>) -> Result<Command> {
    let mut index = default_index_path();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-i" | "--index" => index = expect_path_arg(&mut args, &arg)?,
            "-h" | "--help" => return Ok(Command::Help),
            value => return Err(format!("unknown stats argument: {value}").into()),
        }
    }
    Ok(Command::Stats { index })
}

fn expect_path_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(expect_string_arg(args, flag)?))
}

fn expect_string_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| format!("missing value for {flag}").into())
}

pub fn default_thread_count() -> usize {
    thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(4)
        .clamp(1, 32)
}

fn parse_thread_count(value: &str) -> Result<usize> {
    let threads = value.parse::<usize>()?;
    Ok(if threads == 0 {
        default_thread_count()
    } else {
        threads.min(32)
    })
}

pub fn default_root() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from(r"C:\")
    } else {
        PathBuf::from("/")
    }
}

pub fn default_index_path() -> PathBuf {
    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data)
            .join("quickfind")
            .join("index.qf");
    }

    PathBuf::from(".quickfind").join("index.qf")
}

pub fn print_help() {
    println!(
        "\
quickfind - tiny Everything-like file finder

USAGE:
  quickfind index [ROOTS...] [--output PATH] [--threads N]
  quickfind search <QUERY> [--index PATH] [--limit N]
  quickfind stats [--index PATH]

EXAMPLES:
  quickfind index C:\\Users C:\\Projects
  quickfind search cargo.toml
  quickfind search \"invoice 2026\" --limit 20
"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_args(args: &[&str]) -> Command {
        parse_from(args.iter().map(|value| value.to_string()))
            .unwrap()
            .command
    }

    #[test]
    fn search_accepts_options_after_query() {
        let command = parse_args(&["search", "cargo", "--index", "custom.qf", "--limit", "20"]);

        let Command::Search {
            query,
            index,
            limit,
        } = command
        else {
            panic!("expected search command");
        };

        assert_eq!(query, "cargo");
        assert_eq!(index, PathBuf::from("custom.qf"));
        assert_eq!(limit, 20);
    }

    #[test]
    fn search_collects_multiword_query_around_options() {
        let command = parse_args(&["search", "report", "--limit", "3", "2026"]);

        let Command::Search { query, limit, .. } = command else {
            panic!("expected search command");
        };

        assert_eq!(query, "report 2026");
        assert_eq!(limit, 3);
    }

    #[test]
    fn index_threads_zero_means_auto() {
        let command = parse_args(&["index", ".", "--threads", "0"]);

        let Command::Index { threads, .. } = command else {
            panic!("expected index command");
        };

        assert_eq!(threads, default_thread_count());
    }
}
