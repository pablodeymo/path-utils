use std::ffi::OsStr;
use std::io;
use std::path::Path;

/// Gets the directory path of the current executable
///
/// # Errors
///
/// Returns an error if the current executable path cannot be determined,
/// or if the path has no parent directory.
pub fn path_of_executable() -> io::Result<String> {
    let exe_path = std::env::current_exe()?;
    exe_path
        .parent()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cannot get parent directory"))
}

pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).extension().and_then(OsStr::to_str)
}

/// Gets the name of the file, without the extension
pub fn get_stem_from_filename(filename: &str) -> Option<&str> {
    Path::new(filename).file_stem().and_then(OsStr::to_str)
}

/// Converts the filename extension to lowercase
#[must_use]
pub fn convert_filename_extension_to_lowercase(filename: &str) -> Option<String> {
    let extension = get_extension_from_filename(filename)?;
    let stem = get_stem_from_filename(filename)?;
    Some(format!("{}.{}", stem, extension.to_lowercase()))
}

/// Returns the full path of the first file found in `dir_name` with the given `extension`.
///
/// # Note on trailing quote handling
/// This function also matches files with trailing double-quote characters after the extension
/// (e.g., `file.txt"` or `file.txt""`). This handles edge cases where filenames may contain an erroneous
/// trailing quote from external systems or malformed input data.
///
/// # Errors
///
/// Returns an error if the directory cannot be read, or if no file with
/// the given extension is found.
pub fn get_first_filename_of_directory_with_extension(
    dir_name: &str,
    extension: &str,
) -> io::Result<String> {
    let dir = Path::new(&dir_name);
    if !dir.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Directory not found",
        ));
    }
    for path in std::fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
    {
        // Ignore trailing quotes to handle malformed filenames from external sources
        if let Some(file_ext) = get_extension_from_filename(path.to_str().unwrap_or(""))
            && file_ext.trim_end_matches('"') == extension
        {
            return Ok(path.to_string_lossy().to_string());
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
}

/// Runs a command inside a Docker container.
///
/// Returns the stdout output on success. If the command fails (non-zero exit code),
/// returns an error containing the stderr output.
///
/// # Errors
///
/// Returns an error if the `docker` process cannot be executed, or if the
/// command exits with a non-zero status code.
pub fn run_command_in_docker(
    internal_command: &str,
    container_name: &str,
    network_name: Option<&str>,
    workdir: Option<&str>,
    mount_dir: Option<&str>,
    entrypoint: Option<&str>,
    include_su: bool,
) -> io::Result<String> {
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
        Err(io::Error::other(format!(
            "Docker command failed with exit code {}: {stderr}",
            output.status.code().unwrap_or(-1)
        )))
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
