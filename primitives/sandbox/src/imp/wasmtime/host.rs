//! Host Functions
use super::util;
use crate::{Error, HostFuncType};
use sp_std::fmt;
use wasmtime::{Extern, Func, Store, Val};

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

	pub fn build(self, store: &Store, state: &mut T) -> Result<Vec<Extern>, Error> {
		// self.funcs.iter().map(|v| Func::new(store));
		// Func::wrap(store, |v: Val| {});
		Ok(vec![])
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
