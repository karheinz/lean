extern crate chrono;
extern crate serde;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local, Timelike, Weekday};


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
    pub fn is_valid(&self) -> bool {
        ! self.is_invalid()
    }

    fn is_invalid(&self) -> bool {
        self.title.is_empty() ||
        self.description.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
