//! Colored terminal logger implementation.

use colored::Colorize;

use crate::core::infra::Logger;

/// Logger implementation with colored terminal output.
#[derive(Debug, Clone)]
pub struct ColoredLogger {
    /// Verbosity level (0 = errors only, 1 = info, 2 = debug).
    verbosity: u8,
}

impl ColoredLogger {
    /// Create a new ColoredLogger with the specified verbosity level.
    pub fn new(verbosity: u8) -> Self {
        Self { verbosity }
    }
}

impl Default for ColoredLogger {
    fn default() -> Self {
        Self::new(1)
    }
}

impl Logger for ColoredLogger {
    fn info(&self, msg: &str) {
        if self.verbosity >= 1 {
            println!("{} {}", "[INFO]".blue(), msg);
        }
    }

    fn warn(&self, msg: &str) {
        if self.verbosity >= 1 {
            println!("{} {}", "[WARN]".yellow(), msg);
        }
    }

    fn error(&self, msg: &str) {
        eprintln!("{} {}", "[ERROR]".red(), msg);
    }

    fn debug(&self, msg: &str) {
        if self.verbosity >= 2 {
            println!("{} {}", "[DEBUG]".dimmed(), msg);
        }
    }

    fn success(&self, msg: &str) {
        if self.verbosity >= 1 {
            println!("{} {}", "[OK]".green(), msg);
        }
    }
}
