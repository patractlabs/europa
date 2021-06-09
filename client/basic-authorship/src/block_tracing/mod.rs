use std::{
	collections::HashMap,
	sync::{
		atomic::{AtomicU64, Ordering},
		Arc,
	},
	time::Instant,
};

use parking_lot::Mutex;
use tracing::{
	dispatcher,
	span::{Attributes, Id, Record},
	Dispatch, Level, Subscriber,
};
use tracing_subscriber::CurrentSpan;

use sc_tracing::{SpanDatum, TraceEvent, Values};
use sp_tracing::WASM_TRACE_IDENTIFIER;

mod parser;

pub struct BlockSubscriber {
	pub targets: Vec<(String, Level)>,
	pub next_id: AtomicU64,
	pub current_span: CurrentSpan,
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
			current_span: CurrentSpan::default(),
			spans: Mutex::new(HashMap::new()),
			events: Mutex::new(Vec::new()),
		}
	}
}

// The name of a field required for all events.
const REQUIRED_EVENT_FIELD: &str = "method";

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
		let parent_id = attrs.parent().cloned().or_else(|| self.current_span.id());
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
		let parent_id = event.parent().cloned().or_else(|| self.current_span.id());
		let trace_event = TraceEvent {
			name: event.metadata().name().to_owned(),
			target: event.metadata().target().to_owned(),
			level: *event.metadata().level(),
			values,
			parent_id,
		};
		self.events.lock().push(trace_event);
	}

	fn enter(&self, id: &Id) {
		self.current_span.enter(id.clone());
	}

	fn exit(&self, span: &Id) {
		if self.spans.lock().contains_key(span) {
			self.current_span.exit();
		}
	}
}

pub fn handle_dispatch(dispatch: Dispatch) {
	let block_subscriber = dispatch.downcast_ref::<BlockSubscriber>().expect("fxck");
	let spans: Vec<_> = block_subscriber.spans.lock().drain().collect();
	let events: Vec<_> = block_subscriber.events.lock().drain(..).collect();

	use std::collections::BTreeMap;
	// events into map
	let mut map: BTreeMap<u16, Vec<Event>> = Default::default();

	println!("{:?}", spans);
	println!("{:?}", events);
}

#[derive(Debug,PartialEq)]
pub struct Put {
	key: Vec<u8>,
	value: Option<Vec<u8>>,
}

#[derive(Debug,PartialEq)]
pub struct PutChild {
	child_id: Vec<u8>,
	key: Vec<u8>,
	value: Option<Vec<u8>>,
}

#[derive(Debug,PartialEq)]
pub struct KillChild {
	child_id: Vec<u8>,
}

#[derive(Debug,PartialEq)]
pub struct ClearPrefix {
	prefix: Vec<u8>
}

#[derive(Debug,PartialEq)]
pub struct ClearChildPrefix {
	child_id: Vec<u8>,
	prefix: Vec<u8>
}

#[derive(Debug,PartialEq)]
pub struct Append {
	key: Vec<u8>,
	append: Vec<u8>,
}

#[derive(Debug,PartialEq)]
pub enum Event {
	Put(Put),
	PutChild(PutChild),
	KillChild(KillChild),
	ClearPrefix(ClearPrefix),
	ClearChildPrefix(ClearChildPrefix),
	Append(Append),
  NotConcerned,
}
