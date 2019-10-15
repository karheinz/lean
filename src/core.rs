use chrono::{DateTime, Local, Timelike, Weekday};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::fs::{self, DirBuilder, File};
use std::path::{Path, PathBuf};


pub fn normalize(name: &str) -> String {
    let name = name.trim();
    let re = Regex::new(r"\s+").unwrap();
    re.replace_all(&name, " ").to_string()
}

pub fn to_snake_case(name: &str) -> String {
    let re = Regex::new(r"\s+").unwrap();
    let name = re.replace_all(&name, "_").to_string();
    to_ascii(&name.to_lowercase())
}

pub fn to_ascii(name: &String) -> String {
    let mut tmp: String = String::from(&name[..]);

    for i in [0, 1].iter() {
        for tuple in [("Ä", "Ae"), ("Ö", "Oe"), ("Ü", "Ue"), ("ß", "ss")].iter() {
            let (from, to) = match i % 2 {
                0 => (tuple.0.to_string(), tuple.1.to_string()),
                _ => (tuple.0.to_lowercase(), tuple.1.to_lowercase()),
            };
            let re = Regex::new(&format!("{}", &from)).unwrap();
            tmp = re.replace_all(&tmp, &to[..]).to_string();
        }
    }

    let re = Regex::new(r"[^a-zA-z0-9_-]").unwrap();
    tmp = re.replace_all(&tmp, "").to_string();
    let re = Regex::new(r"__+").unwrap();
    re.replace_all(&tmp, "_").to_string()
}

/// Returns the deepest existing dir in path.
/// At least / (root) is returned.
fn get_deepest_existing_part_of(path: &Path) -> Result<PathBuf, String> {
    let mut hit: Option<PathBuf> = Some(path.to_path_buf());

    // Eeeaaasy!
    if path.exists() {
        if let Ok(full) = path.canonicalize() {
            hit = Some(full);
        }
    // Lets go up step by step and try to resolve path.
    } else {
        let mut abort = false;
        loop {
            hit = match hit.unwrap().parent() {
                Some(parent) => {
                    // Use "." for an empty "" parent, example:
                    //     ("")some/rel/path => (".")some/rel/path
                    if let Some(parent_str) = parent.to_str() {
                        if !parent_str.is_empty() {
                            Some(parent.to_path_buf())
                        } else {
                            Some(PathBuf::from("."))
                        }
                    } else {
                        return Err(format!("malformed path encoding"));
                    }
                },
                _ => None,
            };

            hit = match hit {
                Some(partial) => {
                    match partial.canonicalize() {
                        // Could resolve path: break!
                        Ok(full) => { abort = true; Some(full) },
                        // Couldn't resolve path: continue!
                        Err(_) => Some(partial),
                    }
                },
                _ => None,
            };

            if abort {
                break;
            }
            if let None = hit {
                break;
            }
        }
    }

    // As . and / always exists we always have a hit!
    // The None cases above should never be reached.
    Ok(hit.unwrap())
}

/// Returns current DateTime<Local> with (nano)seconds set to zero.
pub fn now_rounded() -> DateTime<Local> {
    Local::now().with_second(0).unwrap().with_nanosecond(0).unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Recurrence {
    Daily,
    Weekly(Weekday),
    Monthly { week: u32, day: Weekday },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag="type")]
pub enum Occurrence {
    OneTime,
    Periodic { recurrence: Recurrence },
}

/// The Task struct.
///
/// Occurrence can be either OneTime or Periodic.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all="snake_case")]
pub struct Task {
    pub title: String,
    pub description: String,
    //tags: Option<Vec<Tag>>,
    pub occurrence: Occurrence,
    pub effort: Vec<f64>,
    #[serde(default)]
    pub done: f64,
    pub created_at: DateTime<Local>,
    #[serde(default)]
    pub due_at: Option<DateTime<Local>>,
    #[serde(default)]
    pub relates_to: Option<Vec<Task>>,
    #[serde(default)]
    pub depends_on: Option<Vec<Task>>,
    #[serde(default)]
    pub started_at: Option<DateTime<Local>>,
    #[serde(default)]
    pub paused_at: Option<Vec<DateTime<Local>>>,
    #[serde(default)]
    pub resumed_at: Option<Vec<DateTime<Local>>>,
    #[serde(default)]
    pub finished_at: Option<DateTime<Local>>,
    #[serde(default)]
    pub cancelled_at: Option<DateTime<Local>>,
    #[serde(default)]
    pub people: Option<Vec<Person>>,
    //pub notes: Option<Vec<Note>>,
    //pub attachments: Option<Vec<Attachment>>,
}

impl Task {
    pub fn new() -> Task {
        let task = Task {
            title: String::new(),
            description: String::new(),
            created_at: now_rounded(),
            done: 0.0,
            effort: vec![],
            occurrence: Occurrence::OneTime,
            due_at: None,
            relates_to: None,
            depends_on: None,
            started_at: None,
            paused_at: None,
            resumed_at: None,
            finished_at: None,
            cancelled_at: None,
            people: None,
        };

        task
    }

    pub fn is_valid(&self) -> bool {
        ! self.is_invalid()
    }

    fn is_invalid(&self) -> bool {
        self.title.is_empty() ||
        to_snake_case(&self.title).is_empty()
    }

    pub fn done_in_percent(&self) -> u8 {
        (self.done * 100.0) as u8
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub base_dir: PathBuf,
}

impl Workspace {
    pub const CONFIG_FILE: &'static str = ".lean.yaml";
    pub const SUB_DIRS: [&'static str; 8] = ["people", "tasks", "load", "record",
        "views/month", "views/quarter", "views/half_year", "views/year"];

    /// Returns a Workspace object for an already
    /// existing workspace. The passed directory has
    /// to contain or has to be part of a workspace.
    /// An error is returned if this is not the case.
    pub fn new(path: &Path) -> Result<Workspace, String> {
        if !path.is_dir() {
            return Err(format!("{:?} is no directory", path));
        }

        let path = match path.canonicalize() {
            Ok(full) => full,
            Err(reason) => return Err(format!("{}", reason)),
        };

        match Self::lookup_base_dir(&path) {
            Ok(base_dir) => Ok(Workspace { base_dir }),
            Err(reason) => Err(reason),
        }
    }

    /// Returns a Workspace object for a freshly created workspace
    /// in the passed directory. An error is returned if the passed
    /// directory contains or is already part of a workspace.
    pub fn create(path: &Path) -> Result<Workspace, String> {
        let to_check = get_deepest_existing_part_of(path)?;

        if !to_check.is_dir() {
            return Err(format!("{:?} is no directory", to_check));
        }

        if let Ok(_) = Self::lookup_base_dir(&to_check) {
            if path.exists() {
                return Err(format!("directory is (part of) a workspace"));
            } else {
                return Err(format!("directory would be part of a workspace"));
            }
        }

        if let Err(reason) = fs::create_dir_all(&path) {
            return Err(format!("{}", reason));
        }

        let path = match path.canonicalize() {
            Ok(full) => full,
            Err(reason) => return Err(format!("{}", reason)),
        };

        if path.read_dir().unwrap().count() > 0 {
            return Err(format!("directory is not empty"));
        }

        if let Err(reason) = File::create(&path.join(Workspace::CONFIG_FILE)) {
            return Err(format!("{}", reason));
        }

        let mut db = DirBuilder::new();
        db.recursive(true);
        for dir in Workspace::SUB_DIRS.iter() {
            let mut to_create: PathBuf = path.to_path_buf();
            dir.split("/").for_each(|d| { to_create = to_create.join(d); });
            if let Err(reason) = db.create(to_create.as_path()) {
                return Err(format!("{}", reason));
            }
        }

        Ok(Workspace { base_dir: path })
    }

    pub fn add_task(&self, _task: &Task) {
    }

    pub fn get_path(&self, dir: &Option<String>, task: &Task) -> PathBuf {
        let mut path = self.base_dir.join(PathBuf::from("tasks"));
        if let Some(dir) = dir {
            path = path.join(PathBuf::from(dir));
        }
        path.join(self.get_file_name(&task))
    }

    fn get_file_name_prefix(&self, task: &Task) -> String {
        if let Some(time) = task.finished_at {
            format!("X{:?}", time)
        } else if let Some(_) = task.paused_at {
            format!("{:03}S", task.done_in_percent())
        } else if let Some(_) = task.started_at {
            format!("{:03}P", task.done_in_percent())
        } else {
            format!("{:03}U", task.done_in_percent())
        }
    }

    pub fn get_file_name(&self, task: &Task) -> PathBuf {
        PathBuf::from(format!("{}_{}.yaml",
                              self.get_file_name_prefix(&task),
                              to_snake_case(&task.title)))
    }

    /// Looks for (parent) dir which contains CONFIG_FILE (and common dirs).
    fn lookup_base_dir(path: &Path) -> Result<PathBuf, String> {
        let mut current = Some(path);
        let mut hit = false;

        loop {
            current = match current {
                Some(p) => {
                    if p.to_path_buf().join(Workspace::CONFIG_FILE).is_file() {
                        hit = true;
                        Some(p)
                    } else {
                        if let Some(_) = p.parent() {
                            p.parent()
                        } else {
                            None
                        }
                    }
                },
                _ => None,
            };

            if hit || current == None {
                break;
            }
        }

        match current {
            Some(p) => Ok(p.to_path_buf()),
            None => Err(format!("workspace base dir not found")),
        }
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper;
    use mktemp::Temp;


    #[test]
    fn create_task() -> Result<(), String> {
        let mut task = Task {
            title: String::from("Title"),
            description: String::from("Description"),
            created_at: Local::now(),
            done: 0.0,
            effort: vec![5.0],
            occurrence: Occurrence::Periodic {
                recurrence: Recurrence::Weekly(Weekday::Mon) },
            due_at: None,
            relates_to: None,
            depends_on: None,
            started_at: None,
            paused_at: None,
            resumed_at: None,
            finished_at: None,
            cancelled_at: None,
            people: None,
        };

        assert_eq!("Title", task.title);
        assert_eq!("Description", task.description);
        assert_eq!(1, task.effort.len());

        assert!(task.is_valid());
        task.title.clear();
        assert!(! task.is_valid());

        Ok(())
    }

    #[test]
    fn serialize_a_task() -> Result<(), String> {
        let mut task = Task {
            title: String::from("Title"),
            description: String::from("Description"),
            created_at: now_rounded(),
            done: 0.0,
            effort: vec![5.0],
            occurrence: Occurrence::Periodic {
                recurrence: Recurrence::Weekly(Weekday::Mon) },
            due_at: None,
            relates_to: None,
            depends_on: None,
            started_at: None,
            paused_at: None,
            resumed_at: None,
            finished_at: None,
            cancelled_at: None,
            people: None,
        };

        match serde_yaml::to_string(&task) {
            Ok(y) => println!("{}", y),
            Err(reason) => return Err(format!("{}", reason)),
        };

        task.occurrence = Occurrence::OneTime;
        match serde_yaml::to_string(&task) {
            Ok(y) => println!("{}", y),
            Err(reason) => return Err(format!("{}", reason)),
        };

        Ok(())
    }

    #[test]
    fn deserialize_a_task() -> Result<(), String> {
        let task_str = r#"---
title: Title
description: Description
created_at: 2019-10-09T13:00:00+02:00
occurrence:
  type: Periodic
  recurrence:
    monthly:
      week: 3
      day: Fri
effort: [10.0]"#;

        match serde_yaml::from_str::<Task>(task_str) {
            Ok(task) => println!("task: {:?}", task),
            Err(reason) => return Err(format!("{}", reason)),
        }

        Ok(())
    }

    #[test]
    fn serialize_a_task_with_multiline_text() -> Result<(), String> {
        let task = Task {
            title: String::from("Title"),
            description: String::from("Description"),
            created_at: now_rounded(),
            done: 0.0,
            effort: vec![5.0],
            occurrence: Occurrence::Periodic {
                recurrence: Recurrence::Weekly(Weekday::Mon) },
            due_at: None,
            relates_to: None,
            depends_on: None,
            started_at: None,
            paused_at: None,
            resumed_at: None,
            finished_at: None,
            cancelled_at: None,
            people: None,
        };

        match serde_yaml::to_string(&task) {
            Ok(y) => println!("{}", y),
            Err(reason) => return Err(format!("{}", reason)),
        };

        Ok(())
    }

    #[test]
    fn deserialize_a_task_with_multiline_text() -> Result<(), String> {
        let task_str = r#"---
title: Title
description: |
  This
  is a

  multi

  line

  description.
created_at: 2019-10-09T13:00:00+02:00
occurrence:
  type: Periodic
  recurrence:
    monthly:
      week: 3
      day: Fri
effort: [10.0]"#;

        match serde_yaml::from_str::<Task>(task_str) {
            Ok(task) => println!("task: {:?}", task),
            Err(reason) => return Err(format!("{}", reason)),
        }

        Ok(())
    }

    #[test]
    fn lookup_base_dir() -> Result<(), String> {
        let missing = PathBuf::from("/no/lean/workspace");
        if let Ok(_) = Workspace::lookup_base_dir(missing.as_path()) {
            return Err(format!("Error!"))
        }

        let tmp_dir: Temp = match Temp::new_dir() {
            Ok(dir) => dir,
            Err(reason) => return Err(format!("{}", reason)),
        };

        assert!(tmp_dir.as_path().is_dir());

        let tmp_dir_a_b_c = tmp_dir.to_path_buf()
                                   .join("a").join("b").join("c");
        let mut builder = DirBuilder::new();
        builder.recursive(true);

        if let Err(reason) = builder.create(&tmp_dir_a_b_c) {
            return Err(format!("{}", reason));
        }

        if let Err(reason) = File::create(&tmp_dir
                                          .join(Workspace::CONFIG_FILE)) {
            return Err(format!("{}", reason));
        }

        match Workspace::lookup_base_dir(&tmp_dir_a_b_c.as_path()) {
            Ok(path) => assert_eq!(tmp_dir.to_path_buf(), path),
            Err(reason) => return Err(reason),
        };
        match Workspace::lookup_base_dir(&tmp_dir.as_path()) {
            Ok(path) => assert_eq!(tmp_dir.to_path_buf(), path),
            Err(reason) => return Err(reason),
        };

        Ok(())
    }

    #[test]
    fn create_workspace() -> Result<(), String> {
        test_helper::prepare_temp_dir()?;

        let tmp_dir: Temp = match Temp::new_dir() {
            Ok(dir) => dir,
            Err(reason) => return Err(format!("{}", reason)),
        };

        assert!(tmp_dir.as_path().is_dir());
        assert!(!tmp_dir.join(Workspace::CONFIG_FILE).is_file());
        if let Ok(_) = Workspace::new(tmp_dir.as_path()) {
            return Err(format!("object for non existing workspace created"));
        }

        Workspace::create(tmp_dir.as_path())?;
        assert!(tmp_dir.join(Workspace::CONFIG_FILE).is_file());

        for dir in Workspace::SUB_DIRS.iter() {
            let mut to_check: PathBuf = tmp_dir.to_path_buf();
            dir.split("/").for_each(|d| { to_check = to_check.join(d); });
            assert!(to_check.exists());
        }

        Workspace::new(tmp_dir.as_path())?;

        if let Err(reason) = fs::remove_file(tmp_dir.join(Workspace::CONFIG_FILE)) {
            return Err(format!("{}", reason));
        }
        if let Err(reason) = fs::create_dir(tmp_dir.join("some_dir")) {
            return Err(format!("{}", reason));
        }
        if let Ok(_) = Workspace::create(tmp_dir.as_path()) {
            return Err(format!("workspace in non empty directory created"));
        }

        Ok(())
    }
}
