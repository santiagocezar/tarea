#[macro_use]
extern crate lazy_static;
use clap::{AppSettings, Parser, Subcommand};
use dirs::data_dir;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_json::{self, Result};
use std::iter::once;
use std::path::PathBuf;
use std::{
    env, fs,
    io::{self, BufRead, Write},
    process::exit,
};
use uuid::Uuid;

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

#[derive(Parser)]
#[clap(global_setting = AppSettings::DeriveDisplayOrder)]
struct Args {
    #[clap(subcommand)]
    action: Option<Action>,
}

lazy_static! {
    static ref APPDIR: PathBuf = data_dir().unwrap().join("tarea");
    static ref SORT_FILE: PathBuf = APPDIR.join("sort");
}

#[derive(Serialize, Deserialize)]
struct Task {
    id: String,
    task: String,
    done: bool,
    parent: Option<String>,
}

impl Task {
    fn save(&self) -> io::Result<()> {
        let task_file = fs::File::create(APPDIR.join(self.id.clone() + ".json"))?;
        serde_json::to_writer(task_file, self)?;
        Ok(())
    }
    fn remove(&self) -> io::Result<()> {
        fs::remove_file(APPDIR.join(self.id.clone() + ".json"))
    }
}

fn list_tasks() -> io::Result<Vec<Task>> {
    let mut tasks = Vec::new();
    let sort = match fs::File::open(SORT_FILE.as_path()) {
        Ok(sort) => sort,
        Err(e) => {
            if let io::ErrorKind::NotFound = e.kind() {
                return Ok(tasks);
            } else {
                return Err(e);
            }
        }
    };

    for id in io::BufReader::new(sort).lines() {
        let task_file = fs::File::open(APPDIR.join(id? + ".json"))?;
        let task: Task = serde_json::from_reader(task_file)?;
        tasks.push(task);
    }
    Ok(tasks)
}

fn add_task(task: &str) -> io::Result<Task> {
    let id = Uuid::new_v4().to_string();

    let t = Task {
        id,
        task: task.to_string(),
        done: false,
        parent: None,
    };
    t.save()?;

    edit_sort(Edit::Add(&t.id))?;

    Ok(t)
}

enum Edit<'a> {
    None,
    Add(&'a str),
    Del(&'a str),
}

fn edit_sort(edit: Edit) -> io::Result<()> {
    let id = Uuid::new_v4().to_string();
    let mut tasks = list_tasks()?;

    tasks.sort_by_key(|t| t.done);

    let mut sort = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(SORT_FILE.as_path())?;

    if let Edit::Add(id) = edit {
        writeln!(sort, "{}", id)?;
    }
    for task in tasks {
        let skip = if let Edit::Del(id) = edit {
            task.id == id
        } else {
            false
        };
        if !skip {
            writeln!(sort, "{}", task.id)?;
        }
    }

    Ok(())
}

fn get_task(n: usize) -> io::Result<Option<Task>> {
    let mut tasks = list_tasks()?;
    if n >= tasks.len() {
        println!(
            "{}: There's no task number {}",
            "error".bright_red().bold(),
            n + 1
        );
        Ok(None)
    } else {
        Ok(Some(tasks.remove(n)))
    }
}

// Commands

fn command_list_tasks(mark: Option<String>) -> io::Result<()> {
    let tasks = list_tasks()?;
    if tasks.is_empty() {
        println!("{}", "You've got no tasks!".yellow());
    } else {
        for (i, task) in tasks.iter().enumerate() {
            let hl = match &mark {
                None => false,
                Some(id) => id == &task.id,
            };
            print!(" {}", (i + 1).cyan());
            if task.done {
                print!(" {}", "[✓]".green())
            } else {
                print!(" {}", "[ ]".yellow())
            }
            if hl {
                if task.done {
                    println!(" {}", task.task.bold().strikethrough());
                } else {
                    println!(" {}", task.task.bold());
                }
            } else {
                if task.done {
                    println!(" {}", task.task.strikethrough());
                } else {
                    println!(" {}", task.task);
                }
            }
        }
    }
    Ok(())
}

fn command_show_task(n: usize) -> io::Result<()> {
    if let Some(task) = get_task(n)? {
        print!("{}{}", "Task #".dimmed(), (n + 1).dimmed());
        if task.done {
            println!(" - {}", "Done!".green());
        } else {
            println!(" - {}", "Pending".yellow());
        }
        println!("{}", task.task.bold());
    }
    Ok(())
}
fn command_done_task(n: usize, done: bool) -> io::Result<Option<Task>> {
    if let Some(mut task) = get_task(n)? {
        task.done = done;
        task.save()?;
        edit_sort(Edit::None)?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

fn command_remove_task(n: usize) -> io::Result<Option<Task>> {
    if let Some(task) = get_task(n)? {
        edit_sort(Edit::Del(&task.id))?;
        task.remove()?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // create app data directory
    fs::create_dir_all(APPDIR.as_path())?;

    use Action::*;

    if let Some(action) = args.action {
        match action {
            Add { task } => {
                let task_text: String = task.iter().map(|s| s.clone() + " ").collect();
                let task = add_task(task_text.trim_end())?;
                command_list_tasks(Some(task.id))?;
            }
            List => command_list_tasks(None)?,
            Done { id } => {
                let task = command_done_task(id.saturating_sub(1), true)?;
                command_list_tasks(task.map(|t| t.id))?;
            }
            Undo { id } => {
                let task = command_done_task(id.saturating_sub(1), false)?;
                command_list_tasks(task.map(|t| t.id))?;
            }
            Rm { id } => {
                if let Some(task) = command_remove_task(id.saturating_sub(1))? {
                    println!("{} {}", "removed".red(), task.task);
                }
                command_list_tasks(None)?;
            }
        }
    } else {
        command_list_tasks(None)?
    }

    Ok(())
}
