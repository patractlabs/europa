// This file is part of europa

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.
//! `TraceEvent` parser
use super::*;

use nom::{
	branch::alt,
	bytes::{self, complete::tag},
	character::{self, complete::char},
	combinator::map,
	sequence::{delimited, preceded, tuple},
	IResult,
};

struct OptVal(Option<Vec<u8>>);

#[derive(Debug, PartialEq)]
pub struct Message {
	pub id: u16,
	pub event: Event,
}

/// default `Message` if parse failed.
impl Default for Message {
	fn default() -> Self {
		Message {
			id: 0,
			event: Event::NotConcerned,
		}
	}
}

fn parse_message(message: impl AsRef<str>) -> Option<Message> {
	let id = bytes::complete::is_not(":");
	let left = preceded(
		character::complete::char(':'),
		preceded(
			character::complete::multispace0,
			map(bytes::complete::take_till1(|_| false), parse_event),
		),
	);

	let parsed: IResult<_, (&str, _)> = tuple((id, left))(message.as_ref());

	parsed
		.map(|(_, (id, event))| {
			event
				.map(|(_, event)| Message {
					id: u16::from_str_radix(id, 16).unwrap_or_default(),
					event,
				})
				.ok()
		})
		.ok()
		.flatten()
}

fn parse_event(input: &str) -> IResult<&str, Event> {
	alt((
		parse_put_child,
		parse_kill_child,
		parse_clear_prefix,
		parse_clear_child_prefix,
		parse_append,
		parse_not_concerned,
	))(input)
}

/// event parse for `Event::PutChild`
fn parse_put_child(input: &str) -> IResult<&str, Event> {
	let arg = preceded(
		tag("PutChild"),
		delimited(char('('), bytes::complete::is_not(")"), char(')')),
	);

	tuple((arg, character::complete::multispace1, parse_k_equ_opt_v))(input).map(
		|(left, (arg, _, (key, value)))| {
			(
				left,
				Event::PutChild(PutChild {
					child_id: arg.as_bytes().to_vec(),
					key: key.as_bytes().to_vec(),
					value: value.0,
				}),
			)
		},
	)
}

/// event parse for `Event::KillChild`
fn parse_kill_child(input: &str) -> IResult<&str, Event> {
	preceded(
		tag("KillChild"),
		delimited(char('('), bytes::complete::is_not(")"), char(')')),
	)(input)
	.map(|(left, arg)| {
		(
			left,
			Event::KillChild(KillChild {
				child_id: arg.as_bytes().to_vec(),
			}),
		)
	})
}

/// event parse for `Event::ClearPrefix`
fn parse_clear_prefix(input: &str) -> IResult<&str, Event> {
	tuple((
		tag("ClearPrefix"),
		character::complete::multispace1,
		character::complete::hex_digit1,
	))(input)
	.map(|(left, (_, _, value))| {
		(
			left,
			Event::ClearPrefix(ClearPrefix {
				prefix: value.as_bytes().to_vec(),
			}),
		)
	})
}

/// event parse for `Event::ClearChildPrefix`
fn parse_clear_child_prefix(input: &str) -> IResult<&str, Event> {
	let arg = preceded(
		tag("ClearChildPrefix"),
		delimited(char('('), bytes::complete::is_not(")"), char(')')),
	);

	tuple((
		arg,
		character::complete::multispace1,
		character::complete::hex_digit1,
	))(input)
	.map(|(left, (arg, _, value))| {
		(
			left,
			Event::ClearChildPrefix(ClearChildPrefix {
				child_id: arg.as_bytes().to_vec(),
				prefix: value.as_bytes().to_vec(),
			}),
		)
	})
}

/// event parse for `Event::Append`
fn parse_append(input: &str) -> IResult<&str, Event> {
	tuple((
		tag("Append"),
		character::complete::multispace1,
		map(
			// parse all bytes left to k-v
			bytes::complete::take_while(|_| true),
			parse_k_equ_v,
		),
	))(input)
	.map(|(_, (left, _, kv))| match kv {
		Ok((_, (key, value))) => (
			left,
			Event::Append(Append {
				key: key.as_bytes().to_vec(),
				append: value.as_bytes().to_vec(),
			}),
		),

		Err(_) => (left, Event::NotConcerned),
	})
}

/// event parse fallback to `Event::NotConcerned`
fn parse_not_concerned(input: &str) -> IResult<&str, Event> {
	Ok((input, Event::NotConcerned))
}

/// k-v parser, eg. 0000=1111
fn parse_k_equ_v(input: &str) -> IResult<&str, (&str, &str)> {
	let (value, (key, _)) =
		tuple((bytes::complete::take_while1(|c: char| c != '='), tag("=")))(input)?;

	Ok((value, (key, value)))
}

/// k-opt-v parser, eg. 0000=Some(1111)
fn parse_k_equ_opt_v(input: &str) -> IResult<&str, (&str, OptVal)> {
	let (value, (key, _, opt)) = tuple((
		bytes::complete::take_while1(|c: char| c != '='),
		tag("="),
		alt((parse_none, parse_some)),
	))(input)?;

	Ok((value, (key, opt)))
}

fn parse_opt_val(value: impl AsRef<str>) -> Option<Vec<u8>> {
	let parsed: IResult<_, OptVal> = alt((parse_none, parse_some))(value.as_ref());

	parsed.map(|r| r.1 .0).ok().flatten()
}

/// eg. None
fn parse_none(input: &str) -> IResult<&str, OptVal> {
	map(tag("None"), |_: &str| OptVal(None))(input)
}

/// eg. Some(00000bbbbb)
fn parse_some(input: &str) -> IResult<&str, OptVal> {
	preceded(
		tag("Some"),
		delimited(
			char('('),
			map(bytes::complete::is_not(")"), |v: &str| {
				OptVal(Some(v.as_bytes().to_vec()))
			}),
			char(')'),
		),
	)(input)
}

impl From<TraceEvent> for Message {
	fn from(event: TraceEvent) -> Self {
		use parser::*;
		let string_values = event.values.string_values;
		let u64_values = event.values.u64_values;

		// Get & Put
		match string_values.get("method") {
			Some(value) => {
				match &value[..] {
					"Put" => {
						let key = string_values.get("key").unwrap();
						let value = string_values.get("value").unwrap();
						let id = u64_values.get("ext_id").cloned().unwrap_or_default();

						Message {
							id: id as u16,
							event: Event::Put(Put {
								key: key.as_bytes().to_vec(),
								value: parse_opt_val(value),
							}),
						}
					}

					// NB: ignore other methods
					_ => Self::default(),
				}
			}

			None => {
				// Other Event
				match string_values.get("message") {
					Some(value) => parse_message(value).unwrap_or_default(),
					None => Self::default(),
				}
			}
		}
	}
}

#[cfg(test)]
mod tests {
	// use super::super::*;
	// use super::*;
	use crate::block_tracing::parser::{parse_message, parse_opt_val, Message};
	use crate::block_tracing::*;
	use tracing::dispatcher;

	#[test]
	fn test_opt_val_parser() {
		let parsed = parse_opt_val("None");
		assert_eq!(parsed, None);
		let parsed = parse_opt_val("Some()");
		assert_eq!(parsed, None);
		let parsed = parse_opt_val("Some(0000abcd)");
		assert_eq!(parsed, Some(b"0000abcd".to_vec()));
	}

	#[test]
	fn test_message_parser() {
		let parsed = parse_message("0001: PutChild(0002) 0003=Some(0004)");
		assert_eq!(
			parsed,
			Some(Message {
				id: 1,
				event: Event::PutChild(PutChild {
					child_id: b"0002".to_vec(),
					key: b"0003".to_vec(),
					value: Some(b"0004".to_vec()),
				})
			})
		);

		let parsed = parse_message("0001: KillChild(0002)");
		assert_eq!(
			parsed,
			Some(Message {
				id: 1,
				event: Event::KillChild(KillChild {
					child_id: b"0002".to_vec(),
				})
			})
		);

		let parsed = parse_message("0001: ClearPrefix 0002");
		assert_eq!(
			parsed,
			Some(Message {
				id: 1,
				event: Event::ClearPrefix(ClearPrefix {
					prefix: b"0002".to_vec(),
				})
			})
		);

		let parsed = parse_message("0001: ClearChildPrefix(0002) 0003");
		assert_eq!(
			parsed,
			Some(Message {
				id: 1,
				event: Event::ClearChildPrefix(ClearChildPrefix {
					child_id: b"0002".to_vec(),
					prefix: b"0003".to_vec(),
				})
			})
		);

		let parsed = parse_message("533a: Append 0002=0003");
		assert_eq!(
			parsed,
			Some(Message {
				id: 21306,
				event: Event::Append(Append {
					key: b"0002".to_vec(),
					append: b"0003".to_vec(),
				})
			})
		);

		let parsed = parse_message("0001: Append 0002 0003");
		assert_eq!(
			parsed,
			Some(Message {
				id: 1,
				event: Event::NotConcerned
			})
		);
	}

	#[test]
	fn workflow() {
		use codec::Encode;
		use sp_core::hexdisplay::HexDisplay;
		use tracing::trace;
		/// Implement `Encode` by forwarding the stored raw vec.
		struct EncodeOpaqueValue(Vec<u8>);

		impl Encode for EncodeOpaqueValue {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				f(&self.0)
			}
		}

		let global = hack_global_subscriber();
		let dispatch = Dispatch::new(ExtrinsicSubscriber::new("state", global));
		dispatcher::with_default(&dispatch, || -> Result<(), sp_blockchain::Error> {
			let span = tracing::info_span!(
				target: "block_trace",
				"trace_block",
			);
			let _enter = span.enter();
			// trace log
			let id = 1_u16;
			let key = hex::decode("3a65787472696e7369635f696e646578").expect("nothing");
			let value1 = Some(hex::decode("00000000").expect("nothing"));
			let value2: Option<Vec<u8>> = None;
			let value_raw = hex::decode("00010000000500d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d59d9a5dadd47ebda356fb2d0eabfefeee3e2b7aab4dbb6546606ef563f5eb47f00").expect("nothing");
			let value_hash = Some([0_u8; 32]);
			let value_bool = true;

			trace!(
				target: "state",
				method = "Get",
				ext_id = id,
				key = %HexDisplay::from(&key),
				result = ?value1.as_ref().map(HexDisplay::from),
				result_encoded = ?value1.as_ref().map(HexDisplay::from),
			);
			trace!(target: "state", "{:04x}: Hash {}={:?}",
				   id,
				   HexDisplay::from(&key),
				   value_hash,
			);
			trace!(target: "state", "{:04x}: GetChild({}) {}={:?}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
				   value2.as_ref().map(HexDisplay::from)
			);
			trace!(target: "state", "{:04x}: ChildHash({}) {}={:?}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
				   value_hash,
			);
			trace!(target: "state", "{:04x}: Exists {}={:?}",
				   id,
				   HexDisplay::from(&key),
				   value_bool,
			);
			trace!(target: "state", "{:04x}: ChildExists({}) {}={:?}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
				   value_bool,
			);
			trace!(
				target: "state",
				method = "Put",
				ext_id = id,
				key = %HexDisplay::from(&key),
				value = ?value1.as_ref().map(HexDisplay::from),
				value_encoded = %HexDisplay::from(
					&value1
						.as_ref()
						.map(|v| EncodeOpaqueValue(v.clone()))
						.encode()
				),
			);
			trace!(target: "state", "{:04x}: PutChild({}) {}={:?}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
				   value2.as_ref().map(HexDisplay::from)
			);
			trace!(target: "state", "{:04x}: KillChild({})",
				   id,
				   HexDisplay::from(&key),
			);
			trace!(target: "state", "{:04x}: ClearPrefix {}",
				   id,
				   HexDisplay::from(&key),
			);
			trace!(target: "state", "{:04x}: ClearChildPrefix({}) {}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
			);
			trace!(target: "state", "{:04x}: Append {}={}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&value_raw),
			);
			trace!(target: "state", "{:04x}: Root(cached) {}",
				   id,
				   HexDisplay::from(&key),
			);
			trace!(target: "state", "{:04x}: Root {}", id, HexDisplay::from(&key));
			trace!(target: "state", "{:04x}: ChildRoot({}) {}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
			);
			trace!(target: "state", "{:04x}: ChildRoot({}) {}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
			);
			trace!(target: "state", "{:04x}: ChildRoot({})(no_change) {}",
				   id,
				   HexDisplay::from(&key),
				   HexDisplay::from(&key),
			);
			let index = 555_u32;
			let size = 99_u32;
			trace!(
				target: "state",
				"{:04x}: IndexTransaction ({}): {}, {} bytes",
				id,
				index,
				HexDisplay::from(&key),
				size,
			);
			trace!(
				target: "state",
				"{:04x}: RenewTransactionIndex ({}): {}",
				id,
				index,
				HexDisplay::from(&key),
			);
			trace!(
				target: "state",
				"{:04x}: ChangesRoot({})(cached) {:?}",
				id,
				HexDisplay::from(&key),
				value_hash,
			);
			trace!(
				target: "state",
				"Failed to decode changes root parent hash: {}",
				"fxck",
			);
			trace!(
				target: "state",
				"{:04x}: ChangesRoot({}) {:?}",
				id,
				HexDisplay::from(&key),
				value_hash,
			);
			Ok(())
		}).expect("");
		let v = parse(dispatch);
		// must meet Put|PutChild|KillChild|ClearPrefix|ClearChildPrefix|Append
		assert_eq!(v.len(), 6);
		let s = serde_json::to_string_pretty(&v).expect("no expect");
		println!("{:}", s);
	}
}
