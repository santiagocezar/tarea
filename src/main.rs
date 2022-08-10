#[macro_use]
extern crate lazy_static;
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



lazy_static! {
    static ref APPDIR: PathBuf = { data_dir().unwrap().join("sctasks") };
    static ref SORT_FILE: PathBuf = { APPDIR.join("sort") };
}

fn usage(exe: &str) {
    println!(
        "Usage:
    {exe} add [TASK...]         \tAdd a new task
    {exe} list                  \tList tasks to do
    {exe} [TASK ID]             \tShow a task
    {exe} [TASK ID] [SUBCOMMAND]\tTask subcommands:

List of task subcommands:
    {exe} done\tMark task as done
    {exe} undo\tMark task as pending
    {exe} rm  \tRemove task
    " // {exe} [PARENT ID] needs [CHILD ID]\tTurn task into subtask
    )
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

fn add_task(task: String) -> io::Result<Task> {
    let id = Uuid::new_v4().to_string();

    let t = Task {
        id,
        task,
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
        println!("{} {}", "removed".red(), task.task);
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

fn main() -> io::Result<()> {
    // own the args
    let args_vec: Vec<String> = env::args().collect();
    let mut args = args_vec.iter().map(|s| &**s);

    let exe = args.next().unwrap_or("task");

    // create app directory
    fs::create_dir_all(APPDIR.as_path())?;

    // Global subcommands
    match args.next() {
        None => usage(&exe),
        Some("add") => {
            if let Some(arg) = args.next() {
                let task_text: String = args.map(|s| " ".to_string() + &s).collect();
                let task = add_task(arg.to_string() + &task_text)?;
                command_list_tasks(Some(task.id))?;
            } else {
                command_list_tasks(None)?
            }
        }
        Some("list") => command_list_tasks(None)?,
        Some("done") => 
        // Task subcommands
        Some(n) => match usize::from_str_radix(n, 10) {
            Ok(n) => {
                let n = n.saturating_sub(1);
                match args.next() {
                    Some("done") => {
                        let task = command_done_task(n, true)?;
                        command_list_tasks(task.map(|t| t.id))?;
                    }
                    Some("undo") => {
                        let task = command_done_task(n, false)?;
                        command_list_tasks(task.map(|t| t.id))?;
                    }
                    Some("rm") => {
                        let task = command_remove_task(n)?;
                        command_list_tasks(None)?;
                    }
                    Some(_) => usage(&exe),
                    None => command_show_task(n)?,
                }
            }
            _ => usage(&exe),
        },
    }
    Ok(())
}
