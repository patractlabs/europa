use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;

// Declare an instance of the native executor named `Executor`. Not not has wasm part.
native_executor_instance!(
	pub Executor,
	europa_runtime::api::dispatch,
	europa_runtime::native_version,
);
