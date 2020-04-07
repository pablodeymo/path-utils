use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MyError {
    pub msg: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "There is an error")
    }
}

impl Error for MyError {}

pub fn path_of_executable() -> Result<String, Box<dyn Error>> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = format!("{}", exe_path.display());
    let pos = match exe_path_str.rfind('/') {
        Some(valor) => valor,
        None => return Result::Err(Box::new(MyError{msg: "El directorio no contiene /".to_owned()}))
    };

    let ret = match exe_path_str.get(0..pos) {
        Some(valor) => valor,
        None => return Result::Err(Box::new(MyError{msg: "Pos invalida".to_owned()}))
    };
    Ok(ret.to_string())
}

pub fn get_first_filename_of_directory_with_extension(dir_name: &str, extension: &str) -> Result<String, Box<dyn Error>> {
    let dir = std::path::Path::new(&dir_name);
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if let Some(path) = path.to_str() {
                if path.ends_with(&format!(".{}", extension)) || 
                   path.ends_with(&format!(".{}\"", extension)) {
                    return Ok(path.to_string());    
                }
            }
        }
    }
    Result::Err(String::from("File not found").into())
}

pub fn run_command_in_docker(internal_command: &str, container_name: &str,
        network_name: Option<&str>, workdir: Option<&str>,
        mount_dir: Option<&str>, entrypoint: Option<&str>, include_su: bool) -> String {
    use std::process::Command;
    let mut command = Command::new("/usr/bin/docker");

    command.arg("run").arg("--rm");
    if let Some(network_name) = network_name {
        command.arg("--network")
        .arg(network_name);
    }                    
    if let Some(workdir) = workdir {
        command.arg("--workdir")
        .arg(workdir);
    }
    if let Some(entrypoint) = entrypoint {
        command.arg("--entrypoint")
        .arg(entrypoint);
    }
    if let Some(mount_dir) = mount_dir {
        command.arg("-v")
        .arg(mount_dir);
    }

    command.arg(container_name);
    if include_su {
        command.arg("su");
    }
    command.arg("-c")
        .arg(internal_command);
    let execution = command.output().expect("failed to execute process");
    String::from_utf8_lossy(&execution.stdout).to_string()
}
