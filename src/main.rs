use std::env;
use lean::cli::*;


fn parse(args: &[String]) -> Result<Box<dyn Command>, String> {
    let arg1 = args.get(1);

    let mut command_str : Option<&str> = None;
    if let Some(arg1_value) = arg1 {
        command_str = Some(&arg1_value[..]);
    }

    match command_str {
        Some("list") => Ok(Box::new(ListTasks::new(&args[2..])?)),
        Some("show") => Ok(Box::new(ShowTask::new(&args[2..])?)),
        Some(unknown) => Err(format!("unknown command {}", unknown)),
        _ => Err(String::from("unknown command")),
    }
}

fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    parse(&args)?.run()
}
