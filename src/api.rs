use std::{
    fs,
    io::{self, BufRead, Write},
    path::PathBuf,
};

use dirs::data_dir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

lazy_static! {
    pub static ref APPDIR: PathBuf = data_dir().unwrap().join("tarea");
    pub static ref SORT_FILE: PathBuf = APPDIR.join("sort");
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task: String,
    pub done: bool,
    pub parent: Option<String>,
}

impl Task {
    pub fn save(&self) -> io::Result<()> {
        let task_file = fs::File::create(APPDIR.join(self.id.clone() + ".json"))?;
        serde_json::to_writer(task_file, self)?;
        Ok(())
    }
    pub fn remove(&self) -> io::Result<()> {
        fs::remove_file(APPDIR.join(self.id.clone() + ".json"))
    }
}

pub fn list_tasks() -> io::Result<Vec<Task>> {
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

pub fn add_task(task: &str) -> io::Result<Task> {
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

pub enum Edit<'a> {
    None,
    Add(&'a str),
    Del(&'a str),
}

pub fn edit_sort(edit: Edit) -> io::Result<()> {
    let mut tasks = list_tasks()?;

    tasks.sort_by_key(|t| t.done);

    let mut sort = fs::OpenOptions::new()
        .write(true)
        .create(true)
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
