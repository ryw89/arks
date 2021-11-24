use std::ffi::OsStr;
use std::path::Path;

/// Infer whether a file is plain text from its file extension.
pub fn is_plain_text(path: &str) -> bool {
    // First, try mime guess
    match mime_guess::from_path(&path).first_raw() {
        None => {}
        Some(m) => {
            if m.starts_with("text/") {
                return true;
            }
        }
    }

    // Next try other extensions
    let ext = Path::new(path).extension().and_then(OsStr::to_str);
    if ext.is_none() {
        return false;
    }

    let ext = ext.unwrap().to_string();

    let other_text_types = vec!["r"];
    if other_text_types.iter().any(|&x| x == ext) {
        return true;
    }
    false
}
