use lean::cli::*;
use std::env;


fn get_cmd(args: &[String]) -> Option<&str> {
    if let Some(value) = args.get(0) {
        Some(&value[..])
    } else {
        None
    }
}

fn parse(args: &[String]) -> Result<Box<dyn Command>, String> {
    match get_cmd(&args[1..]) {
        Some("help") => Ok(Box::new(ShowHelp::new(&args[0], &args[2..])?)),
        Some("init") => Ok(Box::new(InitWorkspace::new(&args[2..])?)),
        Some("tasks") => {
            match get_cmd(&args[2..]) {
                Some("add") => Ok(Box::new(AddTask::new(&args[3..])?)),
                Some("list") => Ok(Box::new(ListTasks::new(&args[3..])?)),
                Some("show") => Ok(Box::new(ShowTasks::new(&args[3..])?)),
                Some(unknown) => Err(format!("unknown sub command {}", unknown)),
                _ => Err(format!("missing command")),
            }
        },
        Some("people") => {
            match get_cmd(&args[2..]) {
                Some(unknown) => Err(format!("unknown sub command {}", unknown)),
                _ => Err(format!("missing command")),
            }
        },
        Some(unknown) => Err(format!("unknown command {}", unknown)),
        _ => Ok(Box::new(ShowUsage::new(&args[0])?)),
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    match parse(&args) {
        Ok(command) => command.run(),
        Err(reason) => {
            ShowUsage::new(&args[0])?.run()?;
            Err(reason)
        }
    }
}
