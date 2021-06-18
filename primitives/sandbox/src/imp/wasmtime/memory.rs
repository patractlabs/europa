//! Wasmtime memory
use super::util;
use crate::Error;
use sp_std::{ops::Range, slice};
use wasmtime::{Limits, Memory as MemoryRef, MemoryType, Store};

/// Construct a range from an offset to a data length after the offset.
/// Returns None if the end of the range would exceed some maximum offset.
pub fn checked_range(offset: usize, len: usize, max: usize) -> Option<Range<usize>> {
	let end = offset.checked_add(len)?;
	if end <= max {
		Some(offset..end)
	} else {
		None
	}
}

/// Wasmtime memory
#[derive(Clone)]
pub struct Memory {
	store: Store,
	inner: MemoryRef,
}

impl Memory {
	/// New memory with config
	pub fn new(initial: u32, maximum: Option<u32>) -> Result<Memory, Error> {
		let store = util::store_with_dwarf();
		Ok(Memory {
			inner: MemoryRef::new(&store, MemoryType::new(Limits::new(initial, maximum)))
				.expect("init memory fail"),
			store,
		})
	}

	pub fn store(&self) -> &Store {
		&self.store
	}

	pub fn get(&self, ptr: u32, buf: &mut [u8]) -> Result<(), Error> {
		// This should be safe since we don't grow up memory while caching this reference and
		// we give up the reference before returning from this function.
		let memory = unsafe { self.memory_as_slice() };
		let range = checked_range(ptr as usize, buf.len(), memory.len())
			.ok_or_else(|| Error::OutOfBounds)?;
		buf.copy_from_slice(&memory[range]);
		Ok(())
	}

	pub fn set(&self, ptr: u32, buf: &[u8]) -> Result<(), Error> {
		let memory = unsafe { self.memory_as_slice_mut() };
		let range = checked_range(ptr as usize, buf.len(), memory.len())
			.ok_or_else(|| Error::OutOfBounds)?;
		&mut memory[range].copy_from_slice(buf);
		Ok(())
	}

	/// Returns linear memory of the wasm instance as a slice.
	///
	/// # Safety
	///
	/// Wasmtime doesn't provide comprehensive documentation about the exact behavior of the data
	/// pointer. If a dynamic style heap is used the base pointer of the heap can change. Since
	/// growing, we cannot guarantee the lifetime of the returned slice reference.
	unsafe fn memory_as_slice(&self) -> &[u8] {
		let ptr = self.inner.data_ptr() as *const _;
		let len = self.inner.data_size();

		if len == 0 {
			&[]
		} else {
			slice::from_raw_parts(ptr, len)
		}
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

	/// Get the inner memory
	pub fn cast(self) -> MemoryRef {
		self.inner
	}
}
