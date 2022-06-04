// This file is part of europa
//
// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

/// Contract tracing extension.
pub trait ContractTracingDb: Send + Sync {
	fn set_tracing(&mut self, number: u32, index: u32, tracing: String);
}

impl<T: ContractTracingDb + ?Sized> ContractTracingDb for Box<T> {
	fn set_tracing(&mut self, number: u32, index: u32, tracing: String) {
		(&mut **self).set_tracing(number, index, tracing)
	}
}

sp_externalities::decl_extension! {
	/// Extension that supports contract tracing in externalities.
	pub struct ContractTracingDbExt(Box<dyn ContractTracingDb>);
}

impl ContractTracingDbExt {
	/// New instance of contract tracing extension.
	pub fn new<O: ContractTracingDb + 'static>(inner_db: O) -> Self {
		Self(Box::new(inner_db))
	}
}
