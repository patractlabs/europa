// This file is part of europa which is forked form Substrate.

// Copyright (C) 2018-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This crate provides means to instantiate and execute wasm modules.
//!
//! It works even when the user of this library executes from
//! inside the wasm VM. In this case the same VM is used for execution
//! of both the sandbox owner and the sandboxed module, without compromising security
//! and without the performance penalty of full wasm emulation inside wasm.
//!
//! This is achieved by using bindings to the wasm VM, which are published by the host API.
//! This API is thin and consists of only a handful functions. It contains functions for instantiating
//! modules and executing them, but doesn't contain functions for inspecting the module
//! structure. The user of this library is supposed to read the wasm module.
//!
//! When this crate is used in the `std` environment all these functions are implemented by directly
//! calling the wasm VM.
//!
//! Examples of possible use-cases for this library are not limited to the following:
//!
//! - implementing smart-contract runtimes that use wasm for contract code
//! - executing a wasm substrate runtime inside of a wasm parachain
#![warn(missing_docs)]

use sp_std::prelude::*;

pub use sp_core::sandbox::HostError;
pub use sp_wasm_interface::{ReturnValue, Value};

mod imp;

/// add serde function for sp_wasm_interface::ReturnValue & Value;
/// notice it's a hack operation, if ReturnValue, Value are changed, this part should also need change.
pub mod serde_opt_wasm_returnvalue {
	use super::*;
	use serde::{de, ser, Deserialize, Serialize};

	#[derive(Clone, Copy, Serialize, Deserialize)]
	/// A hack struct for `Value`
	pub enum SerdeValue {
		/// A 32-bit integer.
		I32(i32),
		/// A 64-bit integer.
		I64(i64),
		/// A 32-bit floating-point number stored as raw bit pattern.
		///
		/// You can materialize this value using `f32::from_bits`.
		F32(u32),
		/// A 64-bit floating-point number stored as raw bit pattern.
		///
		/// You can materialize this value using `f64::from_bits`.
		F64(u64),
	}
	/// A hack struct for `ReturnValue`
	#[derive(Clone, Copy, Serialize, Deserialize)]
	pub enum SerdeReturnValue {
		/// For returning nothing.
		Unit,
		/// For returning some concrete value.
		Value(SerdeValue),
	}
	impl From<ReturnValue> for SerdeReturnValue {
		fn from(v: ReturnValue) -> Self {
			unsafe { std::mem::transmute::<ReturnValue, SerdeReturnValue>(v) }
		}
	}
	impl Into<ReturnValue> for SerdeReturnValue {
		fn into(self) -> ReturnValue {
			unsafe { std::mem::transmute::<SerdeReturnValue, ReturnValue>(self) }
		}
	}

	/// A serializer that encodes the number as a string
	pub fn serialize<S>(value: &Option<ReturnValue>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: ser::Serializer,
	{
		match value {
			Some(ref value) => {
				let v: SerdeReturnValue = (*value).into();
				v.serialize(serializer)
			}
			None => serializer.serialize_none(),
		}
	}

	/// A deserializer that decodes a string to the number.
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ReturnValue>, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		let data: Option<SerdeReturnValue> = Deserialize::deserialize(deserializer)?;
		Ok(data.map(Into::into))
	}
}

/// Error that can occur while using this crate.
#[derive(
	sp_core::RuntimeDebug, codec::Encode, codec::Decode, serde::Serialize, serde::Deserialize,
)]
pub enum Error {
	/// Module is not valid, couldn't be instantiated.
	Module,

	/// Access to a memory or table was made with an address or an index which is out of bounds.
	///
	/// Note that if wasm module makes an out-of-bounds access then trap will occur.
	OutOfBounds,

	/// Failed to invoke the start function or an exported function for some reason.
	Execution,

	/// WASM inner trap
	Trap(imp::Trap),
}

impl From<Error> for HostError {
	fn from(_e: Error) -> HostError {
		HostError
	}
}

/// Function pointer for specifying functions by the
/// supervisor in [`EnvironmentDefinitionBuilder`].
///
/// [`EnvironmentDefinitionBuilder`]: struct.EnvironmentDefinitionBuilder.html
pub type HostFuncType<T> = fn(&mut T, &[Value]) -> Result<ReturnValue, HostError>;

/// Reference to a sandboxed linear memory, that
/// will be used by the guest module.
///
/// The memory can't be directly accessed by supervisor, but only
/// through designated functions [`get`](Memory::get) and [`set`](Memory::set).
#[derive(Clone)]
pub struct Memory {
	inner: imp::Memory,
}

impl Memory {
	/// Construct a new linear memory instance.
	///
	/// The memory allocated with initial number of pages specified by `initial`.
	/// Minimal possible value for `initial` is 0 and maximum possible is `65536`.
	/// (Since maximum addressable memory is 2<sup>32</sup> = 4GiB = 65536 * 64KiB).
	///
	/// It is possible to limit maximum number of pages this memory instance can have by specifying
	/// `maximum`. If not specified, this memory instance would be able to allocate up to 4GiB.
	///
	/// Allocated memory is always zeroed.
	pub fn new(initial: u32, maximum: Option<u32>) -> Result<Memory, Error> {
		Ok(Memory {
			inner: imp::Memory::new(initial, maximum)?,
		})
	}

	/// Read a memory area at the address `ptr` with the size of the provided slice `buf`.
	///
	/// Returns `Err` if the range is out-of-bounds.
	pub fn get(&self, ptr: u32, buf: &mut [u8]) -> Result<(), Error> {
		self.inner.get(ptr, buf)
	}

	/// Write a memory area at the address `ptr` with contents of the provided slice `buf`.
	///
	/// Returns `Err` if the range is out-of-bounds.
	pub fn set(&self, ptr: u32, value: &[u8]) -> Result<(), Error> {
		self.inner.set(ptr, value)
	}
}

/// Struct that can be used for defining an environment for a sandboxed module.
///
/// The sandboxed module can access only the entities which were defined and passed
/// to the module at the instantiation time.
pub struct EnvironmentDefinitionBuilder<T> {
	inner: imp::EnvironmentDefinitionBuilder<T>,
}

impl<T> EnvironmentDefinitionBuilder<T> {
	/// Construct a new `EnvironmentDefinitionBuilder`.
	pub fn new() -> EnvironmentDefinitionBuilder<T> {
		EnvironmentDefinitionBuilder {
			inner: imp::EnvironmentDefinitionBuilder::new(),
		}
	}

	/// Register a host function in this environment definition.
	///
	/// NOTE that there is no constraints on type of this function. An instance
	/// can import function passed here with any signature it wants. It can even import
	/// the same function (i.e. with same `module` and `field`) several times. It's up to
	/// the user code to check or constrain the types of signatures.
	pub fn add_host_func<N1, N2>(&mut self, module: N1, field: N2, f: HostFuncType<T>)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.inner.add_host_func(module, field, f);
	}

	/// Register a memory in this environment definition.
	pub fn add_memory<N1, N2>(&mut self, module: N1, field: N2, mem: Memory)
	where
		N1: Into<Vec<u8>>,
		N2: Into<Vec<u8>>,
	{
		self.inner.add_memory(module, field, mem.inner);
	}
}

/// Sandboxed instance of a wasm module.
///
/// This instance can be used for invoking exported functions.
pub struct Instance<T> {
	inner: imp::Instance<T>,
}

impl<T> Instance<T> {
	/// Instantiate a module with the given [`EnvironmentDefinitionBuilder`]. It will
	/// run the `start` function (if it is present in the module) with the given `state`.
	///
	/// Returns `Err(Error::Module)` if this module can't be instantiated with the given
	/// environment. If execution of `start` function generated a trap, then `Err(Error::Execution)` will
	/// be returned.
	///
	/// [`EnvironmentDefinitionBuilder`]: struct.EnvironmentDefinitionBuilder.html
	pub fn new(
		code: &[u8],
		env_def_builder: &EnvironmentDefinitionBuilder<T>,
		state: &mut T,
	) -> Result<Instance<T>, Error> {
		Ok(Instance {
			inner: imp::Instance::new(code, &env_def_builder.inner, state)?,
		})
	}

	/// Invoke an exported function with the given name.
	///
	/// # Errors
	///
	/// Returns `Err(Error::Execution)` if:
	///
	/// - An export function name isn't a proper utf8 byte sequence,
	/// - This module doesn't have an exported function with the given name,
	/// - If types of the arguments passed to the function doesn't match function signature
	///   then trap occurs (as if the exported function was called via call_indirect),
	/// - Trap occurred at the execution time.
	pub fn invoke(
		&mut self,
		name: &str,
		args: &[Value],
		state: &mut T,
	) -> Result<ReturnValue, Error> {
		self.inner.invoke(name, args, state)
	}

	/// Get the value from a global with the given `name`.
	///
	/// Returns `Some(_)` if the global could be found.
	pub fn get_global_val(&self, name: &str) -> Option<Value> {
		self.inner.get_global_val(name)
	}
}
