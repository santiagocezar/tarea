use std::{
    collections::HashSet,
    fs,
    io::{self, BufRead, Write},
    path::PathBuf,
};

use dirs::data_dir;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::util::default;

lazy_static! {
    pub static ref APPDIR: PathBuf = data_dir().unwrap().join("tarea");
    pub static ref SORT_FILE: PathBuf = APPDIR.join("sort");
}

#[derive(Serialize, Deserialize, Default, PartialEq, PartialOrd, Eq, Ord, Clone, Copy)]
pub enum State {
    #[default]
    Pending,
    WIP,
    Done,
}

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task: String,
    #[serde(default)]
    pub state: State,
    pub parent: Option<String>,
}

impl Task {
    pub fn new(text: String) -> Self {
        let id = Uuid::new_v4().to_string();

        Self {
            id,
            task: text,
            state: State::Pending,
            parent: None,
        }
    }
    pub fn load(id: &str) -> io::Result<Self> {
        let task_file = fs::File::open(APPDIR.join(id.to_string() + ".json"))?;
        Ok(serde_json::from_reader(task_file)?)
    }
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
        tasks.push(Task::load(&id?)?);
    }
    Ok(tasks)
}

pub enum Edit {
    Add(String),
    Remove(usize),
    Update(usize, State),
}

#[derive(Default)]
pub struct Changeset {
    /// ids of the added tasks
    pub added: HashSet<String>,
    /// ids of the updated tasks
    pub updated: HashSet<String>,
    /// the deleted tasks
    pub deleted: Vec<Task>,
}

pub struct Edits(Vec<Edit>);

impl Edits {
    pub fn does(stuff: impl FnOnce(&mut Self) -> ()) -> io::Result<(Changeset, Vec<Task>)> {
        // run the function to get the edits
        let mut edits = Self(default());
        stuff(&mut edits);

        let Self(edits) = edits;

        // track changes to show feedback to the user
        let mut changeset: Changeset = default();
        let tasks = list_tasks()?;

        if edits.is_empty() {
            // no reason to keep going
            return Ok((changeset, tasks));
        }

        // allows me to "delete" tasks without the items
        // moving all over the vec
        let mut tasks: Vec<_> = tasks.into_iter().map(Some).collect();
        let mut added_tasks: Vec<Task> = default();

        for edit in edits {
            match edit {
                Edit::Add(text) => {
                    let task = Task::new(text);
                    changeset.added.insert(task.id.clone());
                    task.save()?;
                    added_tasks.push(task);
                }
                Edit::Remove(n) => {
                    if n < tasks.len() {
                        if let Some(task) = tasks[n].take() {
                            task.remove()?;
                            changeset.deleted.push(task);
                        }
                    }
                }
                Edit::Update(n, state) => {
                    if n < tasks.len() {
                        if let Some(task) = &mut tasks[n] {
                            changeset.updated.insert(task.id.clone());
                            task.state = state
                        }
                    }
                }
            }
        }

        added_tasks.extend(tasks.into_iter().filter_map(|t| t));
        added_tasks.sort_by_key(|t| t.state);

        let mut sort = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(SORT_FILE.as_path())?;

        for task in &added_tasks {
            writeln!(sort, "{}", task.id)?;
        }
        Ok((changeset, added_tasks))
    }

    pub fn add(&mut self, text: &str) {
        self.0.push(Edit::Add(text.to_string()))
    }
    pub fn remove(&mut self, n: usize) {
        self.0.push(Edit::Remove(n.saturating_sub(1)))
    }
    pub fn update(&mut self, n: usize, state: State) {
        self.0.push(Edit::Update(n.saturating_sub(1), state))
    }
}
