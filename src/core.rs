extern crate serde;

use std::any::Any;
use serde::{Serialize, Deserialize};
use serde::ser::Serializer;


#[derive(Debug,PartialEq,Serialize)]
pub enum Occurrence {
    Daily,
    /// weekday
    Weekly(u32),
    /// week, weekday
    Monthly(u32, u32),
}

#[derive(Debug,PartialEq,Serialize)]
pub enum Duration {
    Minutes(u32),
    Hours(f64),
}

pub trait TaskParams {
    fn as_any(&self) -> &dyn Any;
}

#[derive(Serialize)]
pub struct OneTime {
}

impl TaskParams for OneTime {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Serialize)]
pub struct Periodic {
    pub occurrence: Occurrence,
    pub duration: Duration,
}

impl TaskParams for Periodic {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// The Task struct.
///
/// Can be either OneTime or Periodic.
///
/// # Example
///
/// Specific params can be retrieved like this:
///
/// ```
/// use lean::core::{Task, Periodic, Occurrence, Duration};
///
/// let mut task = Task {
///     title: String::from("Title"),
///     description: String::from("Description"),
///     effort: vec![],
///     params: Box::new(Periodic { occurrence: Occurrence::Weekly(1),
///         duration: Duration::Hours(1.0) }),
///     };
///
/// match task.params::<Periodic>() {
///     Some(params) => {
///         assert_eq!(Occurrence::Weekly(1), params.occurrence);
///         assert_eq!(Duration::Hours(1.0), params.duration);
///     },
///     _ => {}
/// };
/// ```
#[derive(Serialize, Deserialize)]
pub struct Task {
    pub title: String,
    pub description: String,
    //tags: Vec<Tag>,
    pub effort: Vec<f64>,
    #[serde(serialize_with="Task::params_serialize", skip_deserializing)]
    #[serde(default="Task::get_default_params")]
    pub params: Box<dyn TaskParams>,
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
    fn get_default_params() -> Box<dyn TaskParams> {
        Box::new(OneTime {})
    }

    pub fn is_valid(&self) -> bool {
        ! self.is_invalid()
    }

    fn is_invalid(&self) -> bool {
        self.title.is_empty() ||
        self.description.is_empty()
    }

    pub fn params<T>(&self) -> Option<&T>
        where T: 'static {
        self.params.as_any().downcast_ref::<T>()
    }

    fn params_serialize<S>(params: &Box<dyn TaskParams>, s: S)
        -> Result<S::Ok, S::Error>
        where S: Serializer {
        if let Some(one_time) = params.as_any().downcast_ref::<OneTime>() {
            one_time.serialize(s)
        } else if let Some(periodic) = params.as_any().downcast_ref::<Periodic>() {
            periodic.serialize(s)
        } else {
            panic!("unknown TaskParams type")
        }
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
            params: Box::new(Periodic { occurrence: Occurrence::Weekly(1),
                duration: Duration::Hours(1.0) }),
        };

        assert_eq!("Title", task.title);
        assert_eq!("Description", task.description);
        assert_eq!(0, task.effort.len());

        assert!(task.is_valid());
        task.title.clear();
        assert!(! task.is_valid());

        if let None = task.params::<Periodic>() {
            return Err(format!("got no Periodic params"));
        }
        if let Some(_) = task.params::<OneTime>() {
            return Err(format!("got OneTime params instead of Periodic"));
        }

        if let Some(params) = task.params::<Periodic>() {
            assert_eq!(Occurrence::Weekly(1), params.occurrence);
        }

        task.params = Box::new(OneTime {});
        if let None = task.params::<OneTime>() {
            return Err(format!("got no OneTime params"));
        }
        if let Some(_) = task.params::<Periodic>() {
            return Err(format!("got Periodic params instead of OneTime"));
        }

        Ok(())
    }

    #[test]
    fn serialize_a_task() -> Result<(), String> {
        let mut task = Task {
            title: String::from("Title"),
            description: String::from("Description"),
            effort: vec![],
            //params: Box::new(Periodic { occurrence: Occurrence::Weekly(1),
            params: Box::new(Periodic { occurrence: Occurrence::Daily,
                duration: Duration::Hours(1.0) }),
        };

        match serde_yaml::to_string(&task) {
            Ok(y) => println!("{}", y),
            Err(reason) => return Err(format!("{}", reason)),
        };

        task.params = Box::new(OneTime {});
        match serde_yaml::to_string(&task) {
            Ok(y) => println!("{}", y),
            Err(reason) => return Err(format!("{}", reason)),
        };

        Ok(())
    }
}
