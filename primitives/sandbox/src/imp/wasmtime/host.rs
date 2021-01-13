//! Host Functions
use crate::HostFuncType;
use sp_std::fmt;
use wasmtime::{FuncType, Val};

pub struct HostFuncIndex(usize);

pub struct DefinedHostFunctions<T> {
	pub funcs: Vec<HostFuncType<T>>,
}

impl<T> DefinedHostFunctions<T> {
	pub fn new() -> Self {
		Self { funcs: Vec::new() }
	}

	pub fn define(&mut self, f: HostFuncType<T>) -> HostFuncIndex {
		let idx = self.funcs.len();
		self.funcs.push(f);
		HostFuncIndex(idx)
	}
}

impl<T> Clone for DefinedHostFunctions<T> {
	fn clone(&self) -> DefinedHostFunctions<T> {
		DefinedHostFunctions {
			funcs: self.funcs.clone(),
		}
	}
}

#[derive(Debug)]
struct DummyHostError;

impl fmt::Display for DummyHostError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "DummyHostError")
	}
}
