// This file is part of europa

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

pub trait ContractTracingDb: Send + Sync {
	fn set_tracing(&mut self, number: u32, index: u32, tracing: String);
}

impl<T: ContractTracingDb + ?Sized> ContractTracingDb for Box<T> {
	fn set_tracing(&mut self, number: u32, index: u32, tracing: String) {
		(&mut **self).set_tracing(number, index, tracing)
	}
}

sp_externalities::decl_extension! {
	pub struct ContractTracingDbExt(Box<dyn ContractTracingDb>);
}

impl ContractTracingDbExt {
	pub fn new<O: ContractTracingDb + 'static>(inner_db: O) -> Self {
		Self(Box::new(inner_db))
	}
}
