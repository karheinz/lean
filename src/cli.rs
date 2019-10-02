extern crate getopts;

use std::path::{Path, PathBuf};
use getopts::Options;


fn is_dir(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(format!("{:?} is no dir", path))
    }
}

pub trait Command {
    // Runs the command.
    //
    // Returns an error message on failure.
    fn run(&self) -> Result<(), String>;
}

#[derive(Debug)]
pub struct ListTasks {
    dir: PathBuf,
}

impl ListTasks {
    pub fn new(args: &[String]) -> Result<ListTasks, String> {
        let mut options = Options::new();

        options.optopt("d", "dir", "base directory", "DIR");
        match options.parse(&args[..]) {
            Ok(matches) => {
                let mut dir = PathBuf::from(".");
                if let Some(d) = matches.opt_str("dir") {
                    dir = PathBuf::from(d);
                }
                is_dir(dir.as_path())?;

                Ok(ListTasks { dir })
            },
            Err(reason) => Err(format!("{:?}", reason)),
        }
    }
}

impl Command for ListTasks {
    fn run(&self) -> Result<(), String> {
        println!("Here are your tasks!");
        Ok(())
    }
}

#[derive(Debug)]
pub struct ShowTask {
    dir: PathBuf,
    ids: Vec<String>,
}

impl ShowTask {
    pub fn new(args: &[String]) -> Result<ShowTask, String> {
        let mut options = Options::new();

        options.optopt("d", "dir", "base directory", "DIR");
        match options.parse(&args[..]) {
            Ok(matches) => {
                let mut dir = PathBuf::from(".");
                if let Some(d) = matches.opt_str("dir") {
                    dir = PathBuf::from(d);
                }
                is_dir(dir.as_path())?;

                let ids = matches.free;
                if ids.is_empty() {
                    return Err(format!("missing task id(s)"))
                }

                Ok(ShowTask { dir, ids })
            },
            Err(reason) => Err(format!("{:?}", reason)),
        }
    }
}

impl Command for ShowTask {
    fn run(&self) -> Result<(), String> {
        if self.ids.len() == 1 {
            println!("Here is your task {:?}!", self.ids);
        } else {
            println!("Here are your tasks {:?}!", self.ids);
        }

        Ok(())
    }
}
