//! Wasmtime Enviroment
use super::{DefinedHostFunctions, Memory};
use crate::{Error, FunctionType, HostFuncType};
use wasmtime::{Extern, Store};

pub struct EnvironmentDefinitionBuilder<T> {
	pub memory: Option<Memory>,
	pub defined_host_functions: DefinedHostFunctions<T>,
}

impl<T> EnvironmentDefinitionBuilder<T> {
	pub fn new() -> Self {
		EnvironmentDefinitionBuilder {
			memory: None,
			defined_host_functions: DefinedHostFunctions::new(),
		}
	}

	pub fn add_host_func<N1, N2>(
		&mut self,
		_module: N1,
		_field: N2,
		f: HostFuncType<T>,
		sig: FunctionType,
	) where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.defined_host_functions.define(f, sig);
	}

	pub fn add_memory<N1, N2>(&mut self, _module: N1, _field: N2, mem: Memory)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.memory = Some(mem);
	}

	pub fn store(&self) -> Option<&Store> {
		if let Some(memory) = &self.memory {
			Some(memory.store())
		} else {
			None
		}
	}

	pub fn build(&self, store: &Store, state: &mut T) -> Result<Vec<Extern>, Error> {
		let mut imports: Vec<Extern> = vec![];

		// push funcs
		for f in self.defined_host_functions.clone().build(store, state) {
			imports.push(Extern::Func(f));
		}

		// push memory
		if let Some(mem) = &self.memory {
			imports.push(Extern::Memory(mem.clone().cast()));
		}

		Ok(imports)
	}
}
