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
pub struct ShowUsage {
    program: String
}

impl ShowUsage {
    pub fn new(path: &String) -> Result<ShowUsage, String> {
        if let Some(program) = Path::new(path).file_name() {
            if let Some(program_str) = program.to_str() {
                return Ok(ShowUsage { program: String::from(program_str) })
            }
        }

        Err(format!("should never come here"))
    }
}

impl Command for ShowUsage {
    fn run(&self) -> Result<(), String> {
        println!("usage: {} COMMAND [ARGS...]", self.program);
        Ok(())
    }
}

#[derive(Debug)]
pub struct ShowHelp {
    program: String
}

impl ShowHelp {
    pub fn new(path: &String, args: &[String]) -> Result<ShowHelp, String> {
        let options = Options::new();
        match options.parse(&args[..]) {
            Ok(_) => Ok(()),
            Err(reason) => Err(format!("{}", reason)),
        }?;

        check_num_of(&args, 0, 0)?;

        if let Some(program) = Path::new(&path).file_name() {
            if let Some(program_str) = program.to_str() {
                return Ok(ShowHelp { program: String::from(program_str) })
            }
        }

        Err(format!("should never come here"))
    }
}

impl Command for ShowHelp {
    fn run(&self) -> Result<(), String> {
        println!("Help for {} is comming soon ...", self.program);
        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Converts an array of &str elems to a vector of String elems.
    ///
    /// This is usefull as command line arguments will usually come as
    /// a vector of String elems. All command constructors have an args
    /// parameter of that type.
    fn to_args(args: &[&str]) -> Vec<String> {
        let mut args_vector: Vec<String> = Vec::new();

        for arg in args {
            args_vector.push(String::from(*arg));
        }

        args_vector
    }

    fn check_parse_error<T>(result: &Result<T, String>, expected: &str)
        -> Result<(), String> {

        match result {
            Err(reason) => {
                assert_eq!(expected, reason);
                Ok(())
            },
            _ => Err(format!("ignored error: {}", expected))
        }
    }

    #[test]
    fn create_list_tasks_command() -> Result<(), String> {
        let args = to_args(&[]);

        let command = ListTasks::new(&args)?;
        assert_eq!(Path::new("."), command.dir);
        assert_eq!(0, command.limit);

        let args = to_args(&["0"]);

        let command = ListTasks::new(&args)?;
        assert_eq!(Path::new("."), command.dir);
        assert_eq!(0, command.limit);

        Ok(())
    }

    #[test]
    fn create_list_tasks_command_with_limit() -> Result<(), String> {
        let args = to_args(&["10"]);

        let command = ListTasks::new(&args)?;
        assert_eq!(Path::new("."), command.dir);
        assert_eq!(10, command.limit);

        Ok(())
    }

    #[test]
    fn create_list_tasks_command_with_dir() -> Result<(), String> {
        let args = to_args(&["-d", "/tmp"]);

        let command = ListTasks::new(&args)?;
        assert_eq!(Path::new("/tmp"), command.dir);
        assert_eq!(0, command.limit);

        Ok(())
    }

    #[test]
    fn create_list_tasks_command_with_too_many_args() -> Result<(), String> {
        let args = to_args(&["10", "20"]);
        check_parse_error(&ListTasks::new(&args), "too many arguments")
    }
}
