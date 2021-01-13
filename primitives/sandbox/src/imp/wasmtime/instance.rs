//! Wasmtime Instance
use super::EnvironmentDefinitionBuilder;
use crate::{Error, ReturnValue, Value};

pub struct Instance<T> {
	state: T,
}

impl<T> Instance<T> {
	pub fn new(
		code: &[u8],
		env_def_builder: &EnvironmentDefinitionBuilder<T>,
		state: &mut T,
	) -> Result<Instance<T>, Error> {
		todo!()
	}

	pub fn invoke(
		&mut self,
		name: &str,
		args: &[Value],
		state: &mut T,
	) -> Result<ReturnValue, Error> {
		todo!()
	}

	pub fn get_global_val(&self, name: &str) -> Option<Value> {
		todo!()
	}
}
