use chrono::{DateTime, Local, Timelike, Weekday};
use regex::Regex;
use serde::{Serialize, Deserialize};
use std::fs::{self, File};
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
    pub fn new(args: &[String]) -> Result<Task, String> {
        let title = args.join(" ");

        let task = Task {
            title: normalize(&title),
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

        if task.is_valid() {
            Ok(task)
        } else {
            Err(format!("task is invalid"))
        }
    }

    pub fn is_valid(&self) -> bool {
        ! self.is_invalid()
    }

    fn is_invalid(&self) -> bool {
        self.title.is_empty() ||
        to_snake_case(&self.title).is_empty()
    }
}

#[derive(Debug)]
pub struct Workspace {
    pub base_dir: PathBuf,
}

impl Workspace {
    pub const CONFIG_FILE: &'static str = ".lean.yaml";

    /// Returns a Workspace object for an already
    /// existing workspace. The passed directory has
    /// to contain or has to be part of a workspace.
    /// An error is returned if this is not the case.
    pub fn new(path: &Path) -> Result<Workspace, String> {
        if !path.is_dir() {
            return Err(format!("{:?} is no directory", path));
        }

        let path = match fs::canonicalize(path) {
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
        if !path.exists() {
            if let Err(reason) = fs::create_dir_all(&path) {
                return Err(format!("{:?}", reason));
            }
        }

        if !path.is_dir() {
            return Err(format!("{:?} is no directory", path));
        }

        let path = match fs::canonicalize(path) {
            Ok(full) => full,
            Err(reason) => return Err(format!("{}", reason)),
        };

        match Self::lookup_base_dir(&path) {
            Ok(_) => Err(format!("directory is already (part of) a workspace")),
            Err(_) => {
                match File::create(&path.join(Workspace::CONFIG_FILE)) {
                    Ok(_) => Ok(Workspace { base_dir: path }),
                    Err(reason) => Err(format!("{:?}", reason)),
                }
            },
        }
    }

    pub fn add_task(&self, _task: &Task) {
    }

    pub fn get_path(&self, task: &Task) -> PathBuf {
        self.base_dir.join(PathBuf::from(
                format!("{}{}", to_snake_case(&task.title), ".yaml")))
    }

    pub fn get_file_name(&self, task: &Task) -> PathBuf {
        PathBuf::from(self.get_path(&task).file_name().unwrap())
    }

    /// Looks for (parent) dir which contains CONFIG_FILE (and common dirs).
    fn lookup_base_dir(path: &Path) -> Result<PathBuf, String> {
        if !path.is_absolute() {
            return Err(format!("absolute path required for base dir lookup"));
        }

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
    use std::fs::{File, DirBuilder};


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
            Err(reason) => return Err(format!("{:?}", reason)),
        };

        assert!(tmp_dir.as_path().is_dir());

        let tmp_dir_a_b_c = tmp_dir.to_path_buf()
                                   .join("a").join("b").join("c");
        let mut builder = DirBuilder::new();
        builder.recursive(true);

        if let Err(reason) = builder.create(&tmp_dir_a_b_c) {
            return Err(format!("{:?}", reason));
        }

        if let Err(reason) = File::create(&tmp_dir
                                          .join(Workspace::CONFIG_FILE)) {
            return Err(format!("{:?}", reason));
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
            Err(reason) => return Err(format!("{:?}", reason)),
        };

        assert!(tmp_dir.as_path().is_dir());
        assert!(!tmp_dir.join(Workspace::CONFIG_FILE).is_file());
        if let Ok(_) = Workspace::new(tmp_dir.as_path()) {
            return Err(format!("object for non existing workspace created"));
        }

        Workspace::create(tmp_dir.as_path())?;
        assert!(tmp_dir.join(Workspace::CONFIG_FILE).is_file());
        Workspace::new(tmp_dir.as_path())?;

        Ok(())
    }
}
