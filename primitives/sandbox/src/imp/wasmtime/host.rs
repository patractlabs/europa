//! Host Functions
use super::util;
use crate::HostFuncType;
use parity_wasm::elements::FunctionType;
use sp_std::fmt;
use wasmtime::{Func, Store};

pub struct DefinedHostFunctions<T> {
	pub funcs: Vec<(HostFuncType<T>, FunctionType)>,
	// _marker: &std::marker::PhantomData<T>,
}

impl<T> DefinedHostFunctions<T> {
	pub fn new() -> Self {
		Self {
			funcs: Vec::new(),
			// _marker: &std::marker::PhantomData::<T>,
		}
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
			// _marker: &std::marker::PhantomData::<T>,
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
