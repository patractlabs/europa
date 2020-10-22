use sc_cli::{ChainSpec, RuntimeVersion, SubstrateCli};

use crate::cli::Cli;
use crate::{chain_spec, service};

impl SubstrateCli for Cli {
	fn impl_name() -> String {
		"Europa Dev Node".into()
	}

	fn impl_version() -> String {
		env!("SUBSTRATE_CLI_IMPL_VERSION").into()
	}

	fn description() -> String {
		env!("CARGO_PKG_DESCRIPTION").into()
	}

	fn author() -> String {
		env!("CARGO_PKG_AUTHORS").into()
	}

	fn support_url() -> String {
		"https://github.com/patractlabs/europa".into()
	}

	fn copyright_start_year() -> i32 {
		2020
	}

	fn load_spec(&self, id: &str) -> Result<Box<dyn ec_service::ChainSpec>, String> {
		// todo chain_spec would receive some params to generate account or other thing dynamically,
		// maybe use some global vars or something others.
		Ok(match id {
			"dev" => Box::new(chain_spec::development_config()?),
			_ => return Err("Not support this chain-spec".to_string()),
		})
	}

	fn native_runtime_version(_: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
		&europa_runtime::VERSION
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();
	let command = &cli.run;
	let runner = ec_cli::build_runner(&cli, command)?;
	runner.run_node_until_exit(|config| service::new_full(config))
}
