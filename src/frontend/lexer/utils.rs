/// Processes escape sequences in string literals.
///
/// Converts escape sequences like `\n`, `\t`, etc. into their actual
/// character representations.
///
/// # Parameters
/// - `s`: The string with escape sequences to process
///
/// # Returns
/// The string with escape sequences replaced
pub fn process_escapes(s: &str) -> String {
    s.replace("\\n", "\n")
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .replace("\\\\", "\\")
        .replace("\\\"", "\"")
}