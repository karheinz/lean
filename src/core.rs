use std::any::Any;


#[derive(Debug,PartialEq)]
pub enum Occurrence {
    Daily,
    Weekly(u32),
    Monthly(u32, u32),
}

pub trait TaskParams {
    fn as_any(&self) -> &dyn Any;
}

pub struct OneTime {
}

impl TaskParams for OneTime {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct Periodic {
    pub occurrence: Occurrence,
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
/// use lean::core::{Task, Periodic, Occurrence};
///
/// let mut task = Task {
///     title: String::from("Title"),
///     description: String::from("Description"),
///     effort: vec![],
///     params: Box::new(Periodic { occurrence: Occurrence::Weekly(1) }),
///     };
///
/// match task.params::<Periodic>() {
///     Some(params) => assert_eq!(Occurrence::Weekly(1), params.occurrence),
///     _ => {}
/// };
/// ```
pub struct Task {
    pub title: String,
    pub description: String,
    //tags: Vec<Tag>,
    pub effort: Vec<f64>,
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
            params: Box::new(Periodic { occurrence: Occurrence::Weekly(1) }),
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
}
