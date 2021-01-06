//! Sandbox wasmtime implementation
#![cfg(feature = "jit")]
use crate::{Error, HostError, HostFuncType, ReturnValue, Value};
use sp_std::{collections::btree_map::BTreeMap, fmt, mem::transmute};
use wasmtime::{FuncType, Limits, Memory as MemoryRef, MemoryType, Module, Store};

/// This memory is for adapt the wasmi interface of pallet-contract
#[derive(Clone)]
pub struct Memory {
	memref: MemoryRef,
}

impl Memory {
	pub fn new(initial: u32, maximum: Option<u32>) -> Result<Memory, Error> {
		Ok(Memory {
			memref: MemoryRef::new(
				&Store::default(),
				MemoryType::new(Limits::new(initial, maximum)),
			),
		})
	}

	pub fn get(&self, ptr: u32, buf: &mut [u8]) -> Result<(), Error> {
		let idx = ptr as usize;
		unsafe {
			let slice: &[u8] = &self.memref.data_unchecked_mut()[idx..(idx + 1)][..];
			buf.copy_from_slice(slice);
		}
		Ok(())
	}

	pub fn set(&self, ptr: u32, value: &[u8]) -> Result<(), Error> {
		let idx = ptr as usize;
		unsafe {
			self.memref.data_unchecked_mut()[idx..(idx + 1)].copy_from_slice(&value[..]);
		}
		Ok(())
	}
}

struct HostFuncIndex(usize);

struct DefinedHostFunctions<T> {
	funcs: Vec<HostFuncType<T>>,
}

impl<T> Clone for DefinedHostFunctions<T> {
	fn clone(&self) -> DefinedHostFunctions<T> {
		DefinedHostFunctions {
			funcs: self.funcs.clone(),
		}
	}
}

impl<T> DefinedHostFunctions<T> {
	fn new() -> DefinedHostFunctions<T> {
		DefinedHostFunctions { funcs: Vec::new() }
	}

	fn define(&mut self, f: HostFuncType<T>) -> HostFuncIndex {
		let idx = self.funcs.len();
		self.funcs.push(f);
		HostFuncIndex(idx)
	}
}

#[derive(Debug)]
struct DummyHostError;

impl fmt::Display for DummyHostError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "DummyHostError")
	}
}

enum ExternVal {
	HostFunc(HostFuncIndex),
	Memory(Memory),
}

pub struct EnvironmentDefinitionBuilder<T> {
	map: BTreeMap<(Vec<u8>, Vec<u8>), ExternVal>,
	defined_host_functions: DefinedHostFunctions<T>,
}

impl<T> EnvironmentDefinitionBuilder<T> {
	pub fn new() -> EnvironmentDefinitionBuilder<T> {
		EnvironmentDefinitionBuilder {
			map: BTreeMap::new(),
			defined_host_functions: DefinedHostFunctions::new(),
		}
	}

	pub fn add_host_func<N1, N2>(&mut self, module: N1, field: N2, f: HostFuncType<T>)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		let idx = self.defined_host_functions.define(f);
		self.map
			.insert((module.into(), field.into()), ExternVal::HostFunc(idx));
	}

	pub fn add_memory<N1, N2>(&mut self, module: N1, field: N2, mem: Memory)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.map
			.insert((module.into(), field.into()), ExternVal::Memory(mem));
	}
}

pub struct Instance<T> {
	instance: Module,
	defined_host_functions: DefinedHostFunctions<T>,
	_marker: std::marker::PhantomData<T>,
}

impl<T> Instance<T> {
	pub fn new(
		code: &[u8],
		env_def_builder: &EnvironmentDefinitionBuilder<T>,
		state: &mut T,
	) -> Result<Instance<T>, Error> {
		unimplemented!();
		// let module = Module::from_buffer(code)
		// 	.map_err(|_| Error::Module)?
		// 	.try_parse_names();
		// let not_started_instance =
		// 	ModuleInstance::new(&module, env_def_builder).map_err(|_| Error::Module)?;
		//
		// let defined_host_functions = env_def_builder.defined_host_functions.clone();
		// let instance = {
		// 	let mut externals = GuestExternals {
		// 		state,
		// 		defined_host_functions: &defined_host_functions,
		// 	};
		// 	let instance = not_started_instance
		// 		.run_start(&mut externals)
		// 		.map_err(|_| Error::Execution)?;
		// 	instance
		// };
		//
		// Ok(Instance {
		// 	instance,
		// 	defined_host_functions,
		// 	_marker: std::marker::PhantomData::<T>,
		// })
	}

	pub fn invoke(
		&mut self,
		name: &str,
		args: &[Value],
		state: &mut T,
	) -> Result<ReturnValue, Error> {
		unimplemented!();
		// let args = args
		// 	.iter()
		// 	.cloned()
		// 	.map(|v| unsafe {
		// 		let wv: wasmi::RuntimeValue = v.into();
		// 		transmute::<wasmi::RuntimeValue, patract_wasmi::RuntimeValue>(wv)
		// 	})
		// 	.collect::<Vec<_>>();
		// let mut externals = GuestExternals {
		// 	state,
		// 	defined_host_functions: &self.defined_host_functions,
		// };
		// let result = self.instance.invoke_export(&name, &args, &mut externals);
		//
		// match result {
		// 	Ok(None) => Ok(ReturnValue::Unit),
		// 	Ok(Some(val)) => unsafe {
		// 		Ok(ReturnValue::Value(
		// 			transmute::<patract_wasmi::RuntimeValue, wasmi::RuntimeValue>(val).into(),
		// 		))
		// 	},
		// 	Err(e) => Err(Error::WasmExecution(e)),
		// }
	}

	pub fn get_global_val(&self, name: &str) -> Option<Value> {
		None
	}
}
