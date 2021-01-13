//! Instance Imports
use wasmtime::Extern;

/// Imports
pub struct Imports {
	externs: Vec<Extern>,
}
