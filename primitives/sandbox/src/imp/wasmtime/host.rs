//! Host Functions
use super::util;
use crate::HostFuncType;
use parity_wasm::elements::FunctionType;
use sp_std::fmt;
use wasmtime::{Func, Store};

pub struct DefinedHostFunctions<T> {
	pub funcs: Vec<(HostFuncType<T>, FunctionType)>,
}

impl<T> DefinedHostFunctions<T> {
	pub fn new() -> Self {
		Self { funcs: Vec::new() }
	}

	pub fn define(&mut self, f: HostFuncType<T>, sig: FunctionType) {
		self.funcs.push((f, sig));
	}

	pub fn build(self, store: &Store, state: &mut T) -> Vec<Func> {
		let mut funcs = vec![];
		for (f, sig) in self.funcs {
			funcs.push(util::wrap_fn(store, state, f, sig));
		}

		funcs
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
