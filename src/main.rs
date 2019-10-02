extern crate getopts;

use std::env;
use std::path::PathBuf;
use getopts::Options;


pub trait Command {
    // Runs the command.
    //
    // Returns an error message on failure.
    fn run(&self) -> Result<(), String>;
}

#[derive(Debug)]
struct ListTasks {
    dir: PathBuf,
}

impl ListTasks {
    fn new(args: &[String]) -> Result<ListTasks, String> {
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

fn parse(args: &[String]) -> Result<Box<dyn Command>, String> {
    let arg1 = args.get(1);

    let mut command_str : Option<&str> = None;
    if let Some(arg1_value) = arg1 {
        command_str = Some(&arg1_value[..]);
    }

    match command_str {
        Some("list") => {
            match ListTasks::new(&args[2..]) {
                Ok(task) => Ok(Box::new(task)),
                Err(reason) => Err(reason),
            }
        },
        Some(unknown) => Err(format!("unknown command {}", unknown)),
        _ => Err(String::from("unknown command")),
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    match parse(&args) {
        Ok(task) => task.run(),
        Err(reason) => Err(reason),
    }
}
