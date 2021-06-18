// This file is part of europa

// Copyright 2020-2021 patract labs. Licensed under GPL-3.0.

use codec::Codec;
use pallet_contracts_primitives::{Code, ContractExecResult, ContractInstantiateResult};
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait ContractsExtApi<AccountId, Balance, BlockNumber, Hash> where
		AccountId: Codec,
		Balance: Codec,
		BlockNumber: Codec,
		Hash: Codec,
	{
		/// Perform a call from a specified account to a given contract.
		///
		/// See [`pallet_contracts::Pallet::call`].
		fn call(
			origin: AccountId,
			dest: AccountId,
			value: Balance,
			gas_limit: u64,
			input_data: Vec<u8>,
		) -> (ContractExecResult, String);

		/// Instantiate a new contract.
		///
		/// See [`pallet_contracts::Pallet::instantiate`].
		fn instantiate(
			origin: AccountId,
			endowment: Balance,
			gas_limit: u64,
			code: Code<Hash>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> (ContractInstantiateResult<AccountId, BlockNumber>, String);
	}
}
