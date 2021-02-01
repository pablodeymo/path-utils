use anyhow::{anyhow, Result};
use std::ffi::OsStr;
use std::path::Path;

pub fn path_of_executable() -> Result<String> {
    let exe_path = std::env::current_exe()?;
    let exe_path_str = format!("{}", exe_path.display());
    let pos = match exe_path_str.rfind('/') {
        Some(valor) => valor,
        None => return Err(anyhow!("El directorio no contiene /")),
    };

    let ret = match exe_path_str.get(0..pos) {
        Some(valor) => valor,
        None => return Err(anyhow!("Pos invalida")),
    };
    Ok(ret.to_string())
}

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

pub fn get_first_filename_of_directory_with_extension(
    dir_name: &str,
    extension: &str,
) -> Result<String> {
    let dir = Path::new(&dir_name);
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if let Some(path) = path.to_str() {
                if path.ends_with(&format!(".{}", extension))
                    || path.ends_with(&format!(".{}\"", extension))
                {
                    return Ok(path.to_string());
                }
            }
        }
    }
    Err(anyhow!("File not found"))
}

pub fn run_command_in_docker(
    internal_command: &str,
    container_name: &str,
    network_name: Option<&str>,
    workdir: Option<&str>,
    mount_dir: Option<&str>,
    entrypoint: Option<&str>,
    include_su: bool,
) -> String {
    use std::process::Command;
    let mut command = Command::new("/usr/bin/docker");

    command.arg("run").arg("--rm");
    if let Some(network_name) = network_name {
        command.arg("--network").arg(network_name);
    }
    if let Some(workdir) = workdir {
        command.arg("--workdir").arg(workdir);
    }
    if let Some(entrypoint) = entrypoint {
        command.arg("--entrypoint").arg(entrypoint);
    }
    if let Some(mount_dir) = mount_dir {
        command.arg("-v").arg(mount_dir);
    }

    command.arg(container_name);
    if include_su {
        command.arg("su");
    }
    command.arg("-c").arg(internal_command);

    println!("{:?}", command);
    let execution = command.output().expect("failed to execute process");
    String::from_utf8_lossy(&execution.stdout).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension_from_filename() {
        assert_eq!(get_extension_from_filename("abc.gz"), Some("gz"));
    }
}
