// This file is part of europa

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

use std::sync::atomic::AtomicU64;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{
	dispatcher,
	span::{Attributes, Id, Record},
	warn, Dispatch, Level, Subscriber,
};

use sc_tracing::{TraceEvent, Values};
use sp_runtime::{traits::Block as BlockT, SaturatedConversion};
use sp_tracing::WASM_TRACE_IDENTIFIER;

use crate::block_tracing::parser::Message;
use ec_client_api::statekv::StateKv;
use std::sync::Arc;

mod parser;

pub struct ExtrinsicSubscriber {
	pub global: Arc<dyn Subscriber + Send + Sync>,
	pub targets: Vec<(String, Level)>,
	pub next_id: AtomicU64,
	pub events: Mutex<Vec<TraceEvent>>,
}

// Default to TRACE if no level given or unable to parse Level
// We do not support a global `Level` currently
fn parse_target(s: &str) -> (String, Level) {
	match s.find('=') {
		Some(i) => {
			let target = s[0..i].to_string();
			if s.len() > i {
				let level = s[i + 1..].parse::<Level>().unwrap_or(Level::TRACE);
				(target, level)
			} else {
				(target, Level::TRACE)
			}
		}
		None => (s.to_string(), Level::TRACE),
	}
}

impl ExtrinsicSubscriber {
	pub fn new(targets: &str, global: Arc<dyn Subscriber + Send + Sync>) -> Self {
		let next_id = AtomicU64::new(1);
		let mut targets: Vec<_> = targets.split(',').map(parse_target).collect();
		// Ensure that WASM traces are always enabled
		// Filtering happens when decoding the actual target / level
		targets.push((WASM_TRACE_IDENTIFIER.to_owned(), Level::TRACE));
		ExtrinsicSubscriber {
			global,
			targets,
			next_id,
			events: Mutex::new(Vec::new()),
		}
	}
}

impl ExtrinsicSubscriber {
	fn self_enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
		for (target, level) in &self.targets {
			if metadata.level() <= level && metadata.target().starts_with(target) {
				return true;
			}
		}
		false
	}
}

impl Subscriber for ExtrinsicSubscriber {
	fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
		self.global.enabled(metadata) | self.self_enabled(metadata)
	}

	fn new_span(&self, attrs: &Attributes<'_>) -> Id {
		self.global.new_span(attrs)
	}

	fn record(&self, span: &Id, values: &Record<'_>) {
		self.global.record(span, values)
	}

	fn record_follows_from(&self, span: &Id, follows: &Id) {
		self.global.record_follows_from(span, follows)
	}

	fn event(&self, event: &tracing::Event<'_>) {
		if self.self_enabled(event.metadata()) {
			let mut values = Values::default();
			event.record(&mut values);
			let parent_id = event.parent().cloned();
			let trace_event = TraceEvent {
				name: event.metadata().name().to_owned(),
				target: event.metadata().target().to_owned(),
				level: *event.metadata().level(),
				values,
				parent_id,
			};
			self.events.lock().push(trace_event);
		}
		if self.global.enabled(event.metadata()) {
			self.global.event(event)
		}
	}

	fn enter(&self, id: &Id) {
		self.global.enter(id)
	}

	fn exit(&self, _span: &Id) {
		self.global.exit(_span)
	}
}

pub fn hack_global_subscriber() -> Arc<dyn Subscriber + Send + Sync> {
	dispatcher::get_default(|d| {
		// a hack way to get private subscriber in Dispatch to public field.
		pub struct PublicDispatch {
			pub subscriber: Arc<dyn Subscriber + Send + Sync>,
		}
		let pub_dispatch: PublicDispatch = unsafe { std::mem::transmute(d.clone()) };
		pub_dispatch.subscriber
	})
}

pub fn parse(dispatch: Dispatch) -> Vec<Event> {
	let block_subscriber = dispatch
		.downcast_ref::<ExtrinsicSubscriber>()
		.expect("must be same subscriber");
	let events: Vec<_> = block_subscriber.events.lock().drain(..).collect();

	use std::collections::BTreeMap;
	// events into map
	let r = events
		.into_iter()
		.flat_map(|e| {
			let msg: Message = e.into();
			// ignore all unrecognized event
			if let Event::NotConcerned = msg.event {
				None
			} else {
				Some(msg)
			}
		})
		.fold(
			BTreeMap::<u16, Vec<Event>>::new(),
			|mut map, msg: Message| {
				let entry = map.entry(msg.id);
				let v = entry.or_insert(Default::default());
				v.push(msg.event);
				map
			},
		);
	if r.len() != 1 {
		warn!("parse trace meet different Ext instance. Need to modify this part to decide real sequence.");
	}
	// just pick the largest group, if the above warn is printed, need to modify this part.
	r.into_iter().fold(Vec::<Event>::new(), |v, item| {
		if item.1.len() > v.len() {
			item.1
		} else {
			v
		}
	})
}

fn store_result<Block: BlockT, S: StateKv<Block>>(
	events: Vec<Event>,
	number: u64,
	index: u32,
	s: Arc<S>,
) {
	let json = serde_json::to_string(&events).expect("should not failed");
	s.set_extrinsic_changes(number.saturated_into(), index, json)
		.expect("database should not return error.")
}

pub fn handle_dispatch<Block: BlockT, S: StateKv<Block>>(
	dispatch: Dispatch,
	number: u64,
	index: u32,
	s: Arc<S>,
) {
	let events = parse(dispatch);
	store_result::<Block, S>(events, number.saturated_into::<u64>(), index, s)
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Put {
	#[serde(with = "sp_core::bytes")]
	key: Vec<u8>,
	#[serde(with = "serde_helper")]
	value: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct PutChild {
	#[serde(with = "sp_core::bytes")]
	child_id: Vec<u8>,
	#[serde(with = "sp_core::bytes")]
	key: Vec<u8>,
	#[serde(with = "serde_helper")]
	value: Option<Vec<u8>>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct KillChild {
	#[serde(with = "sp_core::bytes")]
	child_id: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ClearPrefix {
	#[serde(with = "sp_core::bytes")]
	prefix: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ClearChildPrefix {
	#[serde(with = "sp_core::bytes")]
	child_id: Vec<u8>,
	#[serde(with = "sp_core::bytes")]
	prefix: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Append {
	#[serde(with = "sp_core::bytes")]
	key: Vec<u8>,
	#[serde(with = "sp_core::bytes")]
	append: Vec<u8>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Event {
	Put(Put),
	PutChild(PutChild),
	KillChild(KillChild),
	ClearPrefix(ClearPrefix),
	ClearChildPrefix(ClearChildPrefix),
	Append(Append),
	NotConcerned,
}

mod serde_helper {
	use serde::{de, ser};
	pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: ser::Serializer,
	{
		match value {
			Some(v) => sp_core::bytes::serialize(v.as_slice(), serializer),
			None => serializer.serialize_none(),
		}
	}

	/// A deserializer that decodes a string to the number.
	pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		let option: Option<sp_core::Bytes> = serde::Deserialize::deserialize(deserializer)?;
		Ok(option.map(|v| v.0))
	}
}
