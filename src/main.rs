use std::env;
use lean::cli::*;


fn parse(args: &[String]) -> Result<Box<dyn Command>, String> {
    let arg1 = args.get(1);

    let mut command_str: Option<&str> = None;
    if let Some(arg1_value) = arg1 {
        command_str = Some(&arg1_value[..]);
    }

    match command_str {
        Some("help") => Ok(Box::new(ShowHelp::new(&args[0], &args[2..])?)),
        Some("add") => Ok(Box::new(AddTask::new(&args[2..])?)),
        Some("list") => Ok(Box::new(ListTasks::new(&args[2..])?)),
        Some("show") => Ok(Box::new(ShowTasks::new(&args[2..])?)),
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
