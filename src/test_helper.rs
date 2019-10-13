use crate::core::Workspace;
use mktemp::Temp;
use std::fs;
use std::path::PathBuf;


/// Removes existing CONFIG_FILE from /tmp dir
/// to avoid (accidental) failed tests.
pub fn prepare_temp_dir() -> Result<(), String> {
    let tmp_dir: PathBuf = match Temp::new_path().release().parent() {
        Some(parent) => parent.to_path_buf(),
        None => PathBuf::from("."),
    };

    if tmp_dir.is_absolute() {
        let config_file = tmp_dir.join(Workspace::CONFIG_FILE);

        if config_file.is_file() {
            if let Err(reason) = fs::remove_file(config_file) {
                return Err(format!("{:?}", reason));
            }
        }
    }

    Ok(())
}
