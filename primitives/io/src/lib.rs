// This file is part of europa

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

#![allow(missing_docs)]

use sp_runtime_interface::runtime_interface;

#[runtime_interface]
pub trait ContractTracing {
	fn store_tracing(&mut self, block: u32, index: u32, tracing: Vec<u8>) {
		use ep_extensions::ContractTracingDbExt;
		use sp_externalities::ExternalitiesExt;
		let tracing = String::from_utf8_lossy(&tracing[..]).to_string();
		self.extension::<ContractTracingDbExt>()
			.expect("set_tracing can be called with ContractTracingDb extension")
			.set_tracing(block, index, tracing);
	}
}
