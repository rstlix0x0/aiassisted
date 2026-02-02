//! Agents domain - compile and manage AI agents

mod commands;
mod compiler;
mod diff;
mod discovery;
mod parser;
mod validator;

pub use commands::{AgentsListCommand, AgentsSetupCommand, AgentsUpdateCommand};
pub use compiler::{compile_agent, CompiledAgent, Platform};
pub use diff::{AgentDiff, AgentDiffer, AgentStatus, AgentsUpdateDiff};
pub use discovery::{AgentDiscovery, AgentInfo};
pub use parser::{parse_agent_md, AgentSpec, Capabilities, ModelTier, ParsedAgent};
pub use validator::{validate_agent, validate_description, validate_name, ValidationError, ValidationResult};
