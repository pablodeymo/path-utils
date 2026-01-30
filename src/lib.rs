use anyhow::{anyhow, Result};
use std::ffi::OsStr;
use std::path::Path;

pub fn path_of_executable() -> Result<String> {
    let exe_path = std::env::current_exe()?;
    exe_path
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| anyhow!("Cannot get parent directory"))
}

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

/// Gets the name of the file, without the extension
pub fn get_stem_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).file_stem().and_then(OsStr::to_str)
}

/// Converts the filename extension to lowercase
pub fn convert_filename_extension_to_lowercase(filename: &str) -> Option<String> {
    let extension = get_extension_from_filename(filename)?;
    let stem = get_stem_from_filename(filename)?;
    Some(format!("{}.{}", stem, extension.to_lowercase()))
}

/// Returns the full path of the first file found in `dir_name` with the given `extension`.
///
/// # Note on trailing quote handling
/// This function also matches files with a trailing double-quote character after the extension
/// (e.g., `file.txt"`). This handles edge cases where filenames may contain an erroneous
/// trailing quote from external systems or malformed input data.
pub fn get_first_filename_of_directory_with_extension(
    dir_name: &str,
    extension: &str,
) -> Result<String> {
    let dir = Path::new(&dir_name);
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if let Some(file_ext) = get_extension_from_filename(path.to_str().unwrap_or("")) {
                // Also check for trailing quote to handle malformed filenames from external sources
                let ext_with_quote = format!("{}\"", extension);
                if file_ext == extension || file_ext == ext_with_quote {
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }
    }
    Err(anyhow!("File not found"))
}

/// Runs a command inside a Docker container.
///
/// Returns the stdout output on success. If the command fails (non-zero exit code),
/// returns an error containing the stderr output.
pub fn run_command_in_docker(
    internal_command: &str,
    container_name: &str,
    network_name: Option<&str>,
    workdir: Option<&str>,
    mount_dir: Option<&str>,
    entrypoint: Option<&str>,
    include_su: bool,
) -> Result<String> {
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

    let output = command.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(anyhow!(
            "Docker command failed with exit code {}: {}",
            output.status.code().unwrap_or(-1),
            stderr
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension_from_filename() {
        assert_eq!(get_extension_from_filename("abc.gz"), Some("gz"));
    }

    #[test]
    fn test_get_stem_from_filename() {
        assert_eq!(get_stem_from_filename("abc.gz"), Some("abc"));
    }

    #[test]
    fn test_convert_to_extension_lowercase() {
        assert_eq!(
            convert_filename_extension_to_lowercase("TEST.XLSX"),
            Some("TEST.xlsx".to_string())
        );
        assert_eq!(
            convert_filename_extension_to_lowercase("TEST.N.XLSX"),
            Some("TEST.N.xlsx".to_string())
        );
    }
}
