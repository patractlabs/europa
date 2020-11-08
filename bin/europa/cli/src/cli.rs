use structopt::StructOpt;

use ec_cli::{RunCmd, StateKvCmd};

#[derive(Debug, StructOpt)]
pub struct Cli {
	#[structopt(flatten)]
	pub run: RunCmd,

	#[structopt(subcommand)]
	pub subcommand: Option<Subcommand>,
}

#[derive(Debug, StructOpt)]
pub enum Subcommand {
	/// print modified stored state kvs for a block.
	StateKv(StateKvCmd),
}
