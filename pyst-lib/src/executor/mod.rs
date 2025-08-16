pub mod context;
pub mod runner;

pub use runner::Executor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitCode {
    Success = 0,
    GenericError = 1,
    CliUsageError = 64,
    ExecutionBlocked = 101,
    NetworkRequired = 102,
    ScriptNotFound = 127,
}

impl From<ExitCode> for i32 {
    fn from(code: ExitCode) -> Self {
        code as i32
    }
}
