//! Wasmtime Instance
use super::{DefinedHostFunctions, EnvironmentDefinitionBuilder};
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
	defined_host_functions: DefinedHostFunctions<T>,
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

		Ok(Instance {
			instance,
			defined_host_functions,
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
			.map(|v| super::util::to_val(v))
			.collect::<Vec<_>>();

		// Externals
		todo!()
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
