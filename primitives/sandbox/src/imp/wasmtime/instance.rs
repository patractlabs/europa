//! Wasmtime Instance
use super::{util, EnvironmentDefinitionBuilder};
use crate::{Error, ReturnValue, Value};
use wasmtime::{Extern, Global, Instance as InstanceRef, Module, Store, Val};

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
		let dummy_store = Store::default();
		let store = if let Some(store) = env_def_builder.store() {
			store
		} else {
			&dummy_store
		};
		let module = Module::from_binary(&store.engine(), code).map_err(|_| Error::Module)?;
		let imports = env_def_builder.build(store, state)?;
		let instance = InstanceRef::new(store, &module, &imports).map_err(|e| {
			println!("{:#?}", e);
			Error::Module
		})?;

		Ok(Instance {
			instance,
			_marker: std::marker::PhantomData::<T>,
		})
	}

	pub fn invoke(
		&mut self,
		name: &str,
		args: &[Value],
		_state: &mut T,
	) -> Result<ReturnValue, Error> {
		let args = args
			.iter()
			.cloned()
			.map(|v| util::to_val(v))
			.collect::<Vec<_>>();

		let func = self.instance.get_func(name).ok_or_else(|| {
			println!("Get function failed");
			Error::Execution
		})?;
		let result = func.call(&args).map_err(|e| {
			println!("{}", e);
			Error::Execution
		})?;

		Ok(util::to_ret_val(if result.len() != 1 {
			println!("the length of result is not correct");
			// return Err(Error::Execution);
			return Ok(ReturnValue::Unit);
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
