#[cfg(feature = "interpreter")]
mod wasmi;
#[cfg(feature = "interpreter")]
pub use self::wasmi::*;

#[cfg(feature = "jit")]
mod wasmtime;
#[cfg(feature = "jit")]
pub use self::wasmtime::*;

/// A trap code describing the reason for a trap.
///
/// All trap instructions have an explicit trap code.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, sp_core::RuntimeDebug)]
enum TrapCode {
	/// The current stack space was exhausted.
	StackOverflow,

	/// An out-of-bounds memory access.
	MemoryOutOfBounds,

	/// A wasm atomic operation was presented with a not-naturally-aligned linear-memory address.
	HeapMisaligned,

	/// An out-of-bounds access to a table.
	TableOutOfBounds,

	/// Indirect call to a null table entry.
	IndirectCallToNull,

	/// Signature mismatch on indirect call.
	BadSignature,

	/// An integer arithmetic operation caused an overflow.
	IntegerOverflow,

	/// An integer division by zero.
	IntegerDivisionByZero,

	/// Failed float-to-int conversion.
	BadConversionToInteger,

	/// Code that was supposed to have been unreachable was reached.
	UnreachableCodeReached,

	/// Execution has potentially run too long and may be interrupted.
	Interrupt,

	/// Host function error
	Host(String),
}

/// Wasm Trap
#[derive(sp_core::RuntimeDebug)]
pub struct Trap {
	code: TrapCode,
	reason: String,
	trace: String,
}