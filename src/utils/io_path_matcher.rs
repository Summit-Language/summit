/// Utility functions for identifying standard library I/O function calls.
pub struct IoPathMatcher;

impl IoPathMatcher {
    /// Checks if a function path matches io::readln or std::io::readln.
    ///
    /// # Parameters
    /// - `path`: The function path components
    ///
    /// # Returns
    /// True if the path matches readln
    pub fn is_readln(path: &[String]) -> bool {
        Self::matches_io_function(path, "readln")
    }

    /// Checks if a function path matches io::read or std::io::read.
    ///
    /// # Parameters
    /// - `path`: The function path components
    ///
    /// # Returns
    /// True if the path matches read
    pub fn is_read(path: &[String]) -> bool {
        Self::matches_io_function(path, "read")
    }
    
    /// Checks if a function path matches io::println or std::io::println.
    ///
    /// # Parameters
    /// - `path`: The function path components
    ///
    /// # Returns
    /// True if the path matches println
    pub(crate) fn is_println(path: &[String]) -> bool {
        (path.len() == 2 && path[0] == "io" && path[1] == "println") ||
            (path.len() == 3 && path[0] == "std" && path[1] == "io" && path[2] == "println")
    }

    /// Checks if a function path matches io::print or std::io::print.
    ///
    /// # Parameters
    /// - `path`: The function path components
    ///
    /// # Returns
    /// True if the path matches print
    pub(crate) fn is_print(path: &[String]) -> bool {
        (path.len() == 2 && path[0] == "io" && path[1] == "print") ||
            (path.len() == 3 && path[0] == "std" && path[1] == "io" && path[2] == "print")
    }
    

    /// Generic helper to match I/O function paths.
    ///
    /// Matches both:
    /// - io::function_name (shorthand)
    /// - std::io::function_name (full path)
    ///
    /// # Parameters
    /// - `path`: The function path components
    /// - `function_name`: The function name to match
    ///
    /// # Returns
    /// True if the path matches the given function name
    fn matches_io_function(path: &[String], function_name: &str) -> bool {
        (path.len() == 2 && path[0] == "io" && path[1] == function_name) ||
            (path.len() == 3 && path[0] == "std" && path[1] == "io" && path[2] == function_name)
    }
}