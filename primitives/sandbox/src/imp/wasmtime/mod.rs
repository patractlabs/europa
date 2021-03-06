//! Wasmtime executor
mod env;
mod instance;
mod memory;
mod util;

// use self::host::DefinedHostFunctions;
pub use self::{env::EnvironmentDefinitionBuilder, instance::Instance, memory::Memory};
