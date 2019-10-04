extern crate serde;

use serde::{Serialize, Deserialize};


#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
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
    //tags: Vec<Tag>,
    pub occurrence: Occurrence,
    pub effort: Vec<f64>,
    //created_at: Time,
    //required_at: Option<Time>,
    //relates_to: Vec<Task>,
    //depends_on: Vec<Task>,
    //started_at: Option<Time>,
    //paused: bool,
    //finished_at: Option<Time>,
    //people: Vec<Person>,
    //notes: Vec<Note>,
    //attachments: Vec<Attachment>,
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
            effort: vec![],
            occurrence: Occurrence::Periodic {
                recurrence: Recurrence::Weekly(Weekday::Monday) },
        };

        assert_eq!("Title", task.title);
        assert_eq!("Description", task.description);
        assert_eq!(0, task.effort.len());

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
            effort: vec![],
            occurrence: Occurrence::Periodic { recurrence:
                Recurrence::Monthly { week: 1, day: Weekday::Friday } },
        };
        match serde_yaml::to_string(&task) {
            Ok(y) => println!("{}", y),
            Err(reason) => return Err(format!("{}", reason)),
        };

        task.occurrence = Occurrence::Periodic { recurrence:
            Recurrence::Weekly(Weekday::Monday) };
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
occurrence:
  type: Periodic
  recurrence:
    monthly:
      week: 3
      day: Friday
effort: []"#;

        match serde_yaml::from_str::<Task>(task_str) {
            Ok(task) => println!("task: {:?}", task),
            Err(reason) => return Err(format!("{}", reason)),
        }

        Ok(())
    }
}
