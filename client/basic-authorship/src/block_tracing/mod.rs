use std::{
	collections::HashMap,
	sync::atomic::{AtomicU64, Ordering},
	time::Instant,
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tracing::{
	span::{Attributes, Id, Record},
	warn, Dispatch, Level, Subscriber,
};

use sc_tracing::{SpanDatum, TraceEvent, Values};
use sp_runtime::{traits::Block as BlockT, SaturatedConversion};
use sp_tracing::WASM_TRACE_IDENTIFIER;

use crate::block_tracing::parser::Message;
use ec_client_api::statekv::StateKv;
use std::sync::Arc;

mod parser;

pub struct BlockSubscriber {
	pub targets: Vec<(String, Level)>,
	pub next_id: AtomicU64,
	pub spans: Mutex<HashMap<Id, SpanDatum>>,
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

impl BlockSubscriber {
	pub fn new(targets: &str) -> Self {
		let next_id = AtomicU64::new(1);
		let mut targets: Vec<_> = targets.split(',').map(parse_target).collect();
		// Ensure that WASM traces are always enabled
		// Filtering happens when decoding the actual target / level
		targets.push((WASM_TRACE_IDENTIFIER.to_owned(), Level::TRACE));
		BlockSubscriber {
			targets,
			next_id,
			spans: Mutex::new(HashMap::new()),
			events: Mutex::new(Vec::new()),
		}
	}
}

impl Subscriber for BlockSubscriber {
	fn enabled(&self, metadata: &tracing::Metadata<'_>) -> bool {
		for (target, level) in &self.targets {
			if metadata.level() <= level && metadata.target().starts_with(target) {
				return true;
			}
		}
		false
	}

	fn new_span(&self, attrs: &Attributes<'_>) -> Id {
		let id = Id::from_u64(self.next_id.fetch_add(1, Ordering::Relaxed));
		let mut values = Values::default();
		attrs.record(&mut values);
		let parent_id = attrs.parent().cloned();
		let span = SpanDatum {
			id: id.clone(),
			parent_id,
			name: attrs.metadata().name().to_owned(),
			target: attrs.metadata().target().to_owned(),
			level: *attrs.metadata().level(),
			line: attrs.metadata().line().unwrap_or(0),
			start_time: Instant::now(),
			values,
			overall_time: Default::default(),
		};

		self.spans.lock().insert(id.clone(), span);
		id
	}

	fn record(&self, span: &Id, values: &Record<'_>) {
		let mut span_data = self.spans.lock();
		if let Some(s) = span_data.get_mut(span) {
			values.record(&mut s.values);
		}
	}

	fn record_follows_from(&self, _span: &Id, _follows: &Id) {
		// Not currently used
		unimplemented!("record_follows_from is not implemented");
	}

	fn event(&self, event: &tracing::Event<'_>) {
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

	fn enter(&self, _id: &Id) {}

	fn exit(&self, _span: &Id) {}
}

pub fn parse(dispatch: Dispatch) -> Vec<Event> {
	let block_subscriber = dispatch.downcast_ref::<BlockSubscriber>().expect("fxck");
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
