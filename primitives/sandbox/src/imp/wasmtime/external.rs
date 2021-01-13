//! Externals
use super::{util, DefinedHostFunctions};
use crate::{HostError, ReturnValue, Value};
use wasmtime::{Trap, Val};

pub struct GuestExternals<'e, T: 'e> {
	state: &'e mut T,
	defined_host_functions: &'e DefinedHostFunctions<T>,
}

impl<'e, T> GuestExternals<'e, T> {
	fn invoke_index(&mut self, index: usize, args: &[Value]) -> Result<Option<Val>, Trap> {
		let result = (self.defined_host_functions.funcs[index])(self.state, &args);
		match result {
			Ok(value) => match value {
				ReturnValue::Value(v) => Ok(Some(util::to_val(v))),
				_ => Ok(None),
			},
			Err(HostError) => Err(Trap::new(format!("Invoke index {} failed", index))),
		}
	}
}
