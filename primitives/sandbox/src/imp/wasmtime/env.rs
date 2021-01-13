//! Wasmtime Enviroment
use super::Memory;
use crate::HostFuncType;
pub struct EnvironmentDefinitionBuilder<T> {
	state: T,
}

impl<T> EnvironmentDefinitionBuilder<T> {
	pub fn new() -> Self {
		todo!()
	}

	pub fn add_host_func<N1, N2>(&mut self, module: N1, field: N2, f: HostFuncType<T>) {
		todo!()
	}

	pub fn add_memory<N1, N2>(&mut self, module: N1, field: N2, mem: Memory) {
		todo!()
	}
}
