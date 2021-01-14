//! Wasmtime Enviroment
use super::{util, DefinedHostFunctions, Memory};
use crate::{Error, FunctionType, HostError, HostFuncType};
use wasmtime::{Extern, ExternType, Func, FuncType, Store, Val, ValType};

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
		self.defined_host_functions.define(f);
	}

	pub fn add_memory<N1, N2>(&mut self, _module: N1, _field: N2, mem: Memory)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.memory = Some(mem);
	}

	pub fn build(self, store: &Store, state: &mut T) -> Result<Vec<Extern>, Error> {
		let mut imports: Vec<Extern> = vec![];

		// push funcs
		let state_ptr = state as *mut T;
		for f in self.defined_host_functions.funcs.iter() {
			imports.push(Extern::Func(Func::wrap(&store, |args: i32| {
				// let args = if let Some(args) = args
				// 	.iter()
				// 	.map(|v| util::from_val(*v))
				// 	.collect::<Option<Vec<_>>>()
				// {
				// 	args
				// } else {
				// 	return Err(HostError);
				// };
				//
				// unsafe { f(*state_ptr, args) }
			})));
		}

		// push memory
		let mem = self.memory.ok_or(Error::Module)?;
		imports.push(Extern::Memory(mem.cast()));

		Ok(vec![])
	}
}
