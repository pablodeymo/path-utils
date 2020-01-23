use std::error::Error;

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
