//! Wasmtime memory
use crate::Error;
use sp_std::slice;
use wasmtime::{Limits, Memory as MemoryRef, MemoryType, Store};

/// Wasmtime memory
#[derive(Clone)]
pub struct Memory {
	inner: MemoryRef,
	store: Store,
}

impl Memory {
	/// New memory
	pub fn new(initial: u32, maximum: Option<u32>) -> Result<Memory, Error> {
		let store = Store::default();
		Ok(Memory {
			inner: MemoryRef::new(&store, MemoryType::new(Limits::new(initial, maximum))),
			store,
		})
	}

	pub fn get(&self, ptr: u32, buf: &mut [u8]) -> Result<(), Error> {
		// self.inner.data_unchecked_mut().copy
		todo!()
	}

	pub fn set(&self, ptr: u32, buf: &[u8]) -> Result<(), Error> {
		todo!()
	}

	/// Returns linear memory of the wasm instance as a slice.
	///
	/// # Safety
	///
	/// See `[memory_as_slice]`. In addition to those requirements, since a mutable reference is
	/// returned it must be ensured that only one mutable and no shared references to memory exists
	/// at the same time.
	unsafe fn memory_as_slice_mut(&self) -> &mut [u8] {
		let ptr = self.inner.data_ptr();
		let len = self.inner.data_size();

		if len == 0 {
			&mut []
		} else {
			slice::from_raw_parts_mut(ptr, len)
		}
	}
}
