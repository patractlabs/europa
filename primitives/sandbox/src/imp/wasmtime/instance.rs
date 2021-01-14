//! Wasmtime Instance
use super::{util, DefinedHostFunctions, EnvironmentDefinitionBuilder};
use crate::{Error, ReturnValue, Value};
use wasmtime::{Engine, Extern, Global, Instance as InstanceRef, Module, Store, Val};

fn extern_global(extern_: &Extern) -> Option<&Global> {
	match extern_ {
		Extern::Global(glob) => Some(glob),
		_ => None,
	}
}

pub struct Instance<T> {
	instance: InstanceRef,
	_marker: std::marker::PhantomData<T>,
}

impl<T> Instance<T> {
	pub fn new(
		code: &[u8],
		env_def_builder: &EnvironmentDefinitionBuilder<T>,
		state: &mut T,
	) -> Result<Instance<T>, Error> {
		let module = Module::from_binary(&Engine::default(), code).map_err(|_| Error::Module)?;
		let instance =
			InstanceRef::new(&Store::default(), &module, &[]).map_err(|_| Error::Module)?;
		let defined_host_functions = env_def_builder.defined_host_functions.clone();
		let state_ptr = state as *mut T;

		Ok(Instance {
			instance,
			_marker: std::marker::PhantomData::<T>,
		})
	}

	pub fn invoke(
		&mut self,
		name: &str,
		args: &[Value],
		state: &mut T,
	) -> Result<ReturnValue, Error> {
		let args = args
			.iter()
			.cloned()
			.map(|v| util::to_val(v))
			.collect::<Vec<_>>();

		let func = self.instance.get_func(name).ok_or(Error::Execution)?;
		let result = func.call(&args).map_err(|_| Error::Execution)?;

		Ok(util::to_ret_val(if result.len() != 1 {
			return Err(Error::Execution);
		} else {
			result[0].to_owned()
		})
		.ok_or(Error::Execution)?)
	}

	pub fn get_global_val(&self, name: &str) -> Option<Value> {
		let global = match self.instance.get_export(name) {
			Some(global) => global,
			None => return None,
		};

		let global = extern_global(&global)
			.ok_or_else(|| format!("`{}` is not a global", name))
			.ok()?;

		match global.get() {
			Val::I32(val) => Some(Value::I32(val)),
			Val::I64(val) => Some(Value::I64(val)),
			Val::F32(val) => Some(Value::F32(val)),
			Val::F64(val) => Some(Value::F64(val)),
			_ => None,
		}
	}
}
