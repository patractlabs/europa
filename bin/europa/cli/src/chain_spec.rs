use serde_json::json;

use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};

use ec_service::ChainType;

use europa_runtime::{AccountId, Signature};
use europa_runtime::{BalancesConfig, ContractsConfig, GenesisConfig, SudoConfig, SystemConfig};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = ec_service::GenericChainSpec<GenesisConfig>;
type AccountPublic = <Signature as Verify>::Signer;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}
/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config() -> Result<ChainSpec, String> {
	Ok(ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Development,
		move || {
			genesis(
				// Sudo account
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				// Pre-funded accounts
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				true,
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		Some(
			json!({
				"ss58Format": europa_runtime::SS58Prefix::get(),
				"tokenDecimals": europa_runtime::TokenDecimals::get(),
				"tokenSymbol": europa_runtime::TokenSymbol::get(),
			})
			.as_object()
			.expect("network properties generation can not fail; qed")
			.to_owned(),
		),
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn genesis(
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: SystemConfig {
			// Add Wasm runtime to storage.
			code: b"".to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		pallet_contracts: ContractsConfig {
			current_schedule: pallet_contracts::Schedule::default().enable_println(enable_println),
		},
		pallet_sudo: SudoConfig {
			// Assign network admin rights.
			key: root_key,
		},
	}
}
