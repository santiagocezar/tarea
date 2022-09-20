#[macro_use]
extern crate lazy_static;

use clap::{AppSettings, Parser, Subcommand};
use owo_colors::OwoColorize;
use std::{fs, io};

mod api;
mod util;

use api::{Edits, State, Task, APPDIR};

#[derive(Subcommand)]
enum Action {
    /// Add a new task
    Add {
        /// What do you have to do?
        task: Vec<String>,
    },
    /// List every task
    List,
    /// Mark task as done
    Done {
        /// The number displayed
        tasks: Vec<usize>,
    },
    /// Start working in task
    Start {
        /// The number displayed
        tasks: Vec<usize>,
    },
    /// Mark task as pending
    Undo {
        /// The number displayed
        tasks: Vec<usize>,
    },
    /// Remove task
    Rm {
        /// The number displayed
        tasks: Vec<usize>,
    },
}

/// Simple but not unique to-do app
#[derive(Parser)]
#[clap(author, version, about, global_setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    #[clap(subcommand)]
    action: Option<Action>,
    /// Show task
    task: Option<usize>,
    #[clap(short, long, alias = "quiet")]
    /// Don't show list after commands
    silent: bool,
}

fn main() -> io::Result<()> {
    let mut args = Args::parse();

    // create app data directory
    fs::create_dir_all(APPDIR.as_path())?;

    use Action::*;

    if let Some(n) = args.task {
        let mut tasks = api::list_tasks()?;
        if n >= tasks.len() {
            eprintln!(
                "{}: There's no task number {}",
                "error".bright_red().bold(),
                n + 1
            );
        } else {
            let task = tasks.remove(n);
            print!("{}{} - ", "Task #".dimmed(), (n + 1).dimmed());
            match task.state {
                State::Done => println!("{}", "Done!".green()),
                State::WIP => println!("{}", "In progress...".yellow()),
                State::Pending => println!("{}", "Pending".dimmed()),
            }
            println!("{}", task.task.bold());
        }
    }

    let action = args.action.unwrap_or(List);

    // store the modified task to highlight it in the list
    let (changeset, tasks) = Edits::does(|e| {
        match action {
            Add { task } => {
                e.add(task.join(" ").trim_end());
            }
            List => {
                args.silent = false;
            }
            Done { tasks } => {
                for n in tasks {
                    e.update(n, api::State::Done);
                }
            }
            Start { tasks } => {
                for n in tasks {
                    e.update(n, api::State::WIP);
                }
            }
            Undo { tasks } => {
                for n in tasks {
                    e.update(n, api::State::Pending);
                }
            }
            Rm { tasks } => {
                for n in tasks {
                    e.remove(n);
                }
            }
        };
    })?;

    for Task { task, .. } in &changeset.deleted {
        println!("{} {}", " rm (-)".red(), task)
    }
    if !args.silent {
        if tasks.is_empty() {
            println!("{}", "You've got no tasks!".yellow());
        } else {
            for (i, task) in tasks.iter().enumerate() {
                let hl = changeset.added.contains(&task.id) || changeset.updated.contains(&task.id);
                print!("{:>3}", (i + 1).cyan());
                match task.state {
                    State::Done => print!(" {}", "(✓)".bright_green().bold()),
                    State::WIP => print!(" {}", "(…)".bright_yellow().bold()),
                    State::Pending => print!(" {}", "( )".dimmed()),
                }
                match task.state {
                    State::Done if hl => println!(" {}", task.task.bold().strikethrough()),
                    State::Done => println!(" {}", task.task.strikethrough()),
                    _ if hl => println!(" {}", task.task.bold()),
                    _ => println!(" {}", task.task),
                }
            }
        }
    }
    Ok(())
}
