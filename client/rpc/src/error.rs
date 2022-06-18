// This file is part of europa
//
// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

use jsonrpsee::core::Error as JsonRpseeError;

use sp_core::Bytes;
use sp_runtime::traits::{Block as BlockT, NumberFor};

use crate::NumberOrHash;

#[derive(Debug, thiserror::Error)]
pub enum EuropaRpcError<B: BlockT> {
	/// Forward to a height which is more then best.
	#[error("Forward to a height which is more then best ['{:?}' vs '{:?}].", .0, .1)]
	InvalidForwardHeight(NumberFor<B>, NumberFor<B>),
	/// Backward to a height which is more then best.
	#[error("Backward to a height which is more then best ['{:?}' vs '{:?}].", .0, .1)]
	InvalidBackwardHeight(NumberFor<B>, NumberFor<B>),
	/// Block number or hash not existed.
	#[error("Block number or hash not existed ['{:?}'].", .0)]
	InvalidBlockId(NumberOrHash<B>),
	/// No state kvs data for this number or hash
	#[error("No state kvs data for this number or hash ['{:?}'].", .0)]
	NoStateKvs(NumberOrHash<B>),
	/// No child state kvs data for this number or hash
	#[error("No child state kvs data for this number or hash ['{:?}']. root:{:?}", .0, .1)]
	NoChildStateKvs(NumberOrHash<B>, Bytes),
	/// No extrinsic for this number or hash
	#[error("No extrinsic for this number or hash ['{:?}']. index:{:}", .0, .1)]
	NoExtrinsic(NumberOrHash<B>, u32),
	#[error("Client error: {}", .0)]
	Client(#[from] Box<dyn std::error::Error + Send + Sync>),
}

impl<B: BlockT> From<EuropaRpcError<B>> for JsonRpseeError
where
	B: Send + Sync + 'static,
{
	fn from(e: EuropaRpcError<B>) -> Self {
		Self::to_call_error(e)
	}
}
pub fn client_err<B: BlockT>(err: sp_blockchain::Error) -> EuropaRpcError<B> {
	EuropaRpcError::Client(Box::new(err))
}
// pub fn client_err<B: BlockT>(err: sp_blockchain::Error) -> EuropaRpcError<B> {
// 	EuropaRpcError::Client(Box::new(err))
// }
//
// pub fn internal<E: ::std::fmt::Debug>(e: E) -> jsonrpc_core::Error {
// 	jsonrpc_core::Error {
// 		code: jsonrpc_core::ErrorCode::InternalError,
// 		message: "Unknown error occurred".into(),
// 		data: Some(format!("{:?}", e).into()),
// 	}
// }
