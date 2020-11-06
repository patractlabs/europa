use structopt::StructOpt;

use ec_cli::RunCmd;

#[derive(Debug, StructOpt)]
pub struct Cli {
	#[structopt(flatten)]
	pub run: RunCmd,
}
