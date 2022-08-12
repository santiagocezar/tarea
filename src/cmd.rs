use owo_colors::OwoColorize;
use std::io;

use crate::api::{
    self, Edit,
    State::{self, *},
    Task,
};

// Commands

pub fn list_tasks(mark: Option<String>) -> io::Result<()> {
    let tasks = api::list_tasks()?;
    if tasks.is_empty() {
        println!("{}", "You've got no tasks!".yellow());
    } else {
        for (i, task) in tasks.iter().enumerate() {
            let hl = match &mark {
                None => false,
                Some(id) => id == &task.id,
            };
            print!(" {}", (i + 1).cyan());
            match task.state {
                Done => print!(" {}", "(✓)".bright_green().bold()),
                WIP => print!(" {}", "(…)".bright_yellow().bold()),
                Pending => print!(" {}", "( )".dimmed()),
            }
            match task.state {
                Done if hl => println!(" {}", task.task.bold().strikethrough()),
                Done => println!(" {}", task.task.strikethrough()),
                _ if hl => println!(" {}", task.task.bold()),
                _ => println!(" {}", task.task),
            }
        }
    }
    Ok(())
}

pub fn show_task(n: usize) -> io::Result<()> {
    if let Some(task) = get_task(n)? {
        print!("{}{} - ", "Task #".dimmed(), (n + 1).dimmed());
        match task.state {
            Done => println!("{}", "Done!".green()),
            WIP => println!("{}", "In progress...".yellow()),
            Pending => println!("{}", "Pending".dimmed()),
        }
        println!("{}", task.task.bold());
    }
    Ok(())
}
pub fn set_task_state(n: usize, state: State) -> io::Result<Option<Task>> {
    if let Some(mut task) = get_task(n)? {
        task.state = state;
        task.save()?;
        api::edit_sort(Edit::None)?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

pub fn remove_task(n: usize) -> io::Result<Option<Task>> {
    if let Some(task) = get_task(n)? {
        api::edit_sort(Edit::Del(&task.id))?;
        task.remove()?;
        Ok(Some(task))
    } else {
        Ok(None)
    }
}

fn get_task(n: usize) -> io::Result<Option<Task>> {
    let mut tasks = api::list_tasks()?;
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
