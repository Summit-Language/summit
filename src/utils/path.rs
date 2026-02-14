use std::env;
use std::path::PathBuf;

/// Figures out where the Summit standard library is located.
///
/// # Returns
/// The path to the standard library directory as a String.
pub fn get_stdlib_path() -> String {
    if let Ok(env_path) = env::var("SUMMIT_STDLIB_PATH") {
        let path = PathBuf::from(env_path);
        if path.exists() {
            return path.canonicalize()
                .unwrap_or(path)
                .to_str()
                .unwrap()
                .to_string();
        }
    }

    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let lib_path = exe_dir.join("../lib");
            if lib_path.exists() {
                return lib_path.canonicalize()
                    .unwrap_or(lib_path)
                    .to_str()
                    .unwrap()
                    .to_string();
            }
        }
    }

    "./libsummit".to_string()
}