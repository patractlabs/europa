use std::sync::Arc;

use jsonrpc_core::{Error, ErrorCode, Result};
use jsonrpc_derive::rpc;

use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, Zero},
};
use sp_utils::mpsc::{tracing_unbounded, TracingUnboundedReceiver, TracingUnboundedSender};

pub enum Message {}

pub struct Europa<C, B> {
	client: Arc<C>,
	sender: TracingUnboundedSender<Message>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> Europa<C, B> {
	/// Create new `Contracts` with the given reference to the client.
	pub fn new(client: Arc<C>) -> (Self, TracingUnboundedReceiver<Message>) {
		let (tx, rx) = tracing_unbounded::<Message>("mpsc_europa_rpc");
		(
			Europa {
				client,
				sender: tx,
				_marker: Default::default(),
			},
			rx,
		)
	}
}

#[rpc]
pub trait EuropaApi<BlockNumber> {
	#[rpc(name = "europa_forwardToHeight")]
	fn forward_to_height(&self, height: BlockNumber) -> Result<()>;

	#[rpc(name = "europa_backwardToHeight")]
	fn backward_to_height(&self, height: BlockNumber) -> Result<()>;
}

impl<C, B, BlockNumber> EuropaApi<BlockNumber> for Europa<C, B>
where
	C: HeaderBackend<B>,
	C: Send + Sync + 'static,
	B: BlockT,
{
	fn forward_to_height(&self, height: BlockNumber) -> Result<()> {
		unimplemented!()
	}

	fn backward_to_height(&self, height: BlockNumber) -> Result<()> {
		unimplemented!()
	}
}
