use assert_matches::assert_matches;
use ep_sandbox::{EnvironmentDefinitionBuilder, Error, HostError, Instance, ReturnValue, Value};

fn execute_sandboxed(code: &[u8], args: &[Value]) -> Result<ReturnValue, HostError> {
	struct State {
		counter: u32,
	}

	fn env_assert(_e: &mut State, args: &[Value]) -> Result<ReturnValue, HostError> {
		if args.len() != 1 {
			return Err(HostError);
		}
		let condition = args[0].as_i32().ok_or_else(|| HostError)?;
		if condition != 0 {
			Ok(ReturnValue::Unit)
		} else {
			Err(HostError)
		}
	}
	fn env_inc_counter(e: &mut State, args: &[Value]) -> Result<ReturnValue, HostError> {
		if args.len() != 1 {
			return Err(HostError);
		}
		let inc_by = args[0].as_i32().ok_or_else(|| HostError)?;
		e.counter += inc_by as u32;
		Ok(ReturnValue::Value(Value::I32(e.counter as i32)))
	}
	/// Function that takes one argument of any type and returns that value.
	fn env_polymorphic_id(_e: &mut State, args: &[Value]) -> Result<ReturnValue, HostError> {
		if args.len() != 1 {
			return Err(HostError);
		}
		Ok(ReturnValue::Value(args[0]))
	}

	let mut state = State { counter: 0 };

	let mut env_builder = EnvironmentDefinitionBuilder::new();
	env_builder.add_host_func("env", "assert", env_assert);
	env_builder.add_host_func("env", "inc_counter", env_inc_counter);
	env_builder.add_host_func("env", "polymorphic_id", env_polymorphic_id);

	let mut instance = Instance::new(code, &env_builder, &mut state)?;
	let result = instance.invoke("call", args, &mut state);

	result.map_err(|_| HostError)
}

#[test]
fn invoke_args() {
	let code = wat::parse_str(
		r#"
		(module
			(import "env" "assert" (func $assert (param i32)))

			(func (export "call") (param $x i32) (param $y i64)
				;; assert that $x = 0x12345678
				(call $assert
					(i32.eq
						(get_local $x)
						(i32.const 0x12345678)
					)
				)

				(call $assert
					(i64.eq
						(get_local $y)
						(i64.const 0x1234567887654321)
					)
				)
			)
		)
		"#,
	)
	.unwrap();

	let result = execute_sandboxed(
		&code,
		&[Value::I32(0x12345678), Value::I64(0x1234567887654321)],
	);
	assert!(result.is_ok());
}

#[test]
fn return_value() {
	let code = wat::parse_str(
		r#"
		(module
			(func (export "call") (param $x i32) (result i32)
				(i32.add
					(get_local $x)
					(i32.const 1)
				)
			)
		)
		"#,
	)
	.unwrap();

	let return_val = execute_sandboxed(&code, &[Value::I32(0x1336)]).unwrap();
	assert_eq!(return_val, ReturnValue::Value(Value::I32(0x1337)));
}

#[test]
fn signatures_dont_matter() {
	let code = wat::parse_str(
		r#"
		(module
			(import "env" "polymorphic_id" (func $id_i32 (param i32) (result i32)))
			(import "env" "polymorphic_id" (func $id_i64 (param i64) (result i64)))
			(import "env" "assert" (func $assert (param i32)))

			(func (export "call")
				;; assert that we can actually call the "same" function with different
				;; signatures.
				(call $assert
					(i32.eq
						(call $id_i32
							(i32.const 0x012345678)
						)
						(i32.const 0x012345678)
					)
				)
				(call $assert
					(i64.eq
						(call $id_i64
							(i64.const 0x0123456789abcdef)
						)
						(i64.const 0x0123456789abcdef)
					)
				)
			)
		)
		"#,
	)
	.unwrap();

	let return_val = execute_sandboxed(&code, &[]).unwrap();
	assert_eq!(return_val, ReturnValue::Unit);
}

#[test]
fn cant_return_unmatching_type() {
	fn env_returns_i32(_e: &mut (), _args: &[Value]) -> Result<ReturnValue, HostError> {
		Ok(ReturnValue::Value(Value::I32(42)))
	}

	let mut env_builder = EnvironmentDefinitionBuilder::new();
	env_builder.add_host_func("env", "returns_i32", env_returns_i32);

	let code = wat::parse_str(
		r#"
		(module
			;; It's actually returns i32, but imported as if it returned i64
			(import "env" "returns_i32" (func $returns_i32 (result i64)))

			(func (export "call")
				(drop
					(call $returns_i32)
				)
			)
		)
		"#,
	)
	.unwrap();

	// It succeeds since we are able to import functions with types we want.
	let mut instance = Instance::new(&code, &env_builder, &mut ()).unwrap();

	// But this fails since we imported a function that returns i32 as if it returned i64.
	assert_matches!(instance.invoke("call", &[], &mut ()), Err(Error::Execution));
}
