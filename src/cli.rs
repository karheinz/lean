extern crate getopts;

use std::path::PathBuf;
use getopts::Options;


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
