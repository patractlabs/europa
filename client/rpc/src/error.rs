use jsonrpc_core as rpc;

use sp_core::Bytes;
use sp_runtime::traits::Block as BlockT;

use crate::{NumberOf, NumberOrHash};

#[derive(Debug)]
pub enum EuropaRpcError<B: BlockT> {
	InvalidForwardHeight(NumberOf<B>, NumberOf<B>),
	InvalidBackwardHeight(NumberOf<B>, NumberOf<B>),
	InvalidBlockNumber(NumberOf<B>),
	NoStateKvs(NumberOrHash<B>),
	NoChildStateKvs(NumberOrHash<B>, Bytes),
	Client(Box<dyn std::error::Error + Send>),
}

impl<B: BlockT> From<EuropaRpcError<B>> for rpc::Error {
	fn from(e: EuropaRpcError<B>) -> Self {
		match e {
			EuropaRpcError::InvalidForwardHeight(forward, best) => rpc::Error {
				code: rpc::ErrorCode::InvalidParams,
				message: format!(
					"forward height should more than current best: forward: {}|best: {}",
					forward, best
				)
				.into(),
				data: None,
			},
			EuropaRpcError::InvalidBackwardHeight(backward, best) => rpc::Error {
				code: rpc::ErrorCode::InvalidParams,
				message: format!(
					"backward height should less than current best: backward: {}|best: {}",
					backward, best
				)
				.into(),
				data: None,
			},
			EuropaRpcError::InvalidBlockNumber(num) => rpc::Error {
				code: rpc::ErrorCode::InvalidParams,
				message: format!("invalid or not existed block number: {}", num).into(),
				data: None,
			},
			EuropaRpcError::NoStateKvs(num_or_hash) => rpc::Error {
				code: rpc::ErrorCode::InvalidParams,
				message: format!("No state kvs for this block: {:?}", num_or_hash).into(),
				data: None,
			},
			EuropaRpcError::NoChildStateKvs(num_or_hash, bytes) => rpc::Error {
				code: rpc::ErrorCode::InvalidParams,
				message: format!(
					"No child state kvs for this block: {:?}|child:0x{:}",
					num_or_hash,
					hex::encode(&*bytes)
				)
				.into(),
				data: None,
			},
			e => internal(e),
		}
	}
}

pub fn client_err<B: BlockT>(err: sp_blockchain::Error) -> EuropaRpcError<B> {
	EuropaRpcError::Client(Box::new(err))
}

pub fn internal<E: ::std::fmt::Debug>(e: E) -> jsonrpc_core::Error {
	jsonrpc_core::Error {
		code: jsonrpc_core::ErrorCode::InternalError,
		message: "Unknown error occurred".into(),
		data: Some(format!("{:?}", e).into()),
	}
}
