mod api;
mod cmd;

#[macro_use]
extern crate lazy_static;
use clap::{AppSettings, Parser, Subcommand};
use owo_colors::OwoColorize;
use std::{fs, io};

use api::APPDIR;

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
        /// Task number
        id: usize,
    },
    /// Mark task as pending
    Undo {
        /// Task number
        id: usize,
    },
    /// Remove task
    Rm {
        /// Task number
        id: usize,
    },
}

/// Simple but not unique to-do app
#[derive(Parser)]
#[clap(author, version, about, global_setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    #[clap(subcommand)]
    action: Option<Action>,
    #[clap(short, long, alias = "quiet")]
    /// Don't show list after commands
    silent: bool,
}

fn main() -> io::Result<()> {
    let mut args = Args::parse();

    // create app data directory
    fs::create_dir_all(APPDIR.as_path())?;

    use Action::*;

    let action = args.action.unwrap_or(List);

    // store the modified task to highlight it in the list
    let mark = match action {
        Add { task } => {
            let mut task_text = String::new();
            for mut t in task {
                t += " ";
                task_text += &t;
            }
            Some(api::add_task(task_text.trim_end())?)
        }
        Done { id } => cmd::done_task(id.saturating_sub(1), true)?,
        Undo { id } => cmd::done_task(id.saturating_sub(1), false)?,
        Rm { id } => {
            let task = cmd::remove_task(id.saturating_sub(1))?;
            if let Some(task) = &task {
                println!("{} {}", "removed".red(), task.task);
            }
            task
        }
        List => {
            args.silent = false;
            None
        },
    };

    if !args.silent {
        // pass the task id to highlight
        cmd::list_tasks(mark.map(|t| t.id))
    } else {
        Ok(())
    }
}
