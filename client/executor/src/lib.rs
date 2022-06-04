// This file is part of europa
//
// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

use std::{
	marker::PhantomData,
	panic::{AssertUnwindSafe, UnwindSafe},
};

use codec::{Decode, Encode};
use sc_executor::{error::Error, with_externalities_safe, NativeExecutionDispatch};
use sp_core::{
	traits::{CodeExecutor, Externalities, ReadRuntimeVersion, RuntimeCode},
	NativeOrEncoded,
};
use sp_version::{GetNativeVersion, NativeVersion};

/// A generic `CodeExecutor` implementation that uses a delegate to determine wasm code equivalence
/// and dispatch to native code when possible, falling back on `WasmExecutor` when not.
pub struct NativeExecutor<D> {
	/// Dummy field to avoid the compiler complaining about us not using `D`.
	_dummy: PhantomData<D>,
	/// Native runtime version info.
	native_version: NativeVersion,
}

impl<D: NativeExecutionDispatch> Clone for NativeExecutor<D> {
	fn clone(&self) -> Self {
		NativeExecutor { _dummy: Default::default(), native_version: D::native_version() }
	}
}

impl<D: NativeExecutionDispatch> Default for NativeExecutor<D> {
	fn default() -> Self {
		Self::new()
	}
}

impl<D: NativeExecutionDispatch> NativeExecutor<D> {
	/// Create new instance.
	pub fn new() -> Self {
		NativeExecutor { _dummy: Default::default(), native_version: D::native_version() }
	}
}

impl<D: NativeExecutionDispatch + 'static> CodeExecutor for NativeExecutor<D> {
	type Error = Error;

	fn call<
		R: Decode + Encode + PartialEq,
		NC: FnOnce() -> Result<R, Box<dyn std::error::Error + Send + Sync>> + UnwindSafe,
	>(
		&self,
		ext: &mut dyn Externalities,
		_runtime_code: &RuntimeCode,
		method: &str,
		data: &[u8],
		_use_native: bool,
		native_call: Option<NC>,
	) -> (Result<NativeOrEncoded<R>, Self::Error>, bool) {
		let mut ext = AssertUnwindSafe(ext);
		let result = if let Some(call) = native_call {
			with_externalities_safe(&mut **ext, call)
				.and_then(|r| r.map(NativeOrEncoded::Native).map_err(Error::ApiError))
		} else {
			with_externalities_safe(&mut **ext, move || D::dispatch(method, data)).and_then(|r| {
				r.map(NativeOrEncoded::Encoded).ok_or_else(|| Error::MethodNotFound(method.into()))
			})
		};

		(result, true)
	}
}

impl<D: NativeExecutionDispatch> ReadRuntimeVersion for NativeExecutor<D> {
	fn read_runtime_version(&self, _: &[u8], _: &mut dyn Externalities) -> Result<Vec<u8>, String> {
		unimplemented!("Not required in Europa.")
	}
}

impl<D: NativeExecutionDispatch> GetNativeVersion for NativeExecutor<D> {
	fn native_version(&self) -> &NativeVersion {
		&self.native_version
	}
}
