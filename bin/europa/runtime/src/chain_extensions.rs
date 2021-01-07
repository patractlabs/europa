//! Europa Chain Extension
#![cfg_attr(not(feature = "std"), no_std)]
use pallet_contracts::chain_extension::{
	ChainExtension, Environment, Ext, InitState, RetVal, SysConfig, UncheckedFrom,
};
use sp_runtime::DispatchError;

/// The chain Extension of Europa
pub struct EuropaExt;

impl ChainExtension for EuropaExt {
	fn call<E: Ext>(_func_id: u32, _env: Environment<E, InitState>) -> Result<RetVal, DispatchError>
	where
		<E::T as SysConfig>::AccountId: UncheckedFrom<<E::T as SysConfig>::Hash> + AsRef<[u8]>,
	{
		// TODO add other libs
		Ok(RetVal::Converging(0))
	}

	fn enabled() -> bool {
		true
	}
}
