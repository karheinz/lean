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

fn check_num_of(elems: &[String], min: u32, max: u32) -> Result<(), String> {
    assert!(min <= max);

    let count: u32 = elems.len() as u32;

    if count < min {
        Err(format!("too few arguments"))
    } else if count > max {
        Err(format!("too many arguments"))
    } else {
        Ok(())
    }
}

fn to_limit(string: Option<String>, radix: u32) -> Result<u32, String> {
    match string {
        Some(s) => {
            match u32::from_str_radix(&s, radix) {
                Ok(n) => Ok(n),
                Err(e) => Err(format!("{}", e)),
            }
        },
        _ => Ok(0),
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
    limit: u32,
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

                let mut args = matches.free;
                check_num_of(&args, 0, 1)?;

                let limit = to_limit(args.pop(), 10)?;

                Ok(ListTasks { dir, limit })
            },
            Err(reason) => Err(format!("{}", reason)),
        }
    }
}

impl Command for ListTasks {
    fn run(&self) -> Result<(), String> {
        match self.limit {
            0 => println!("Here are your latest tasks!"),
            1 => println!("Here is your latest task!"),
            _ => println!("Here are your latest {} tasks!", self.limit),
        }

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
            Err(reason) => Err(format!("{}", reason)),
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
