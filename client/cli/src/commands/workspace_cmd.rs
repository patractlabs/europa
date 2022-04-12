// This file is part of europa

// Copyright 2020-2022 Patract Labs. Licensed under GPL-3.0.

use ansi_term::{Color, Style};
use structopt::StructOpt;

use sc_cli::SubstrateCli;

use ec_service::BasePath;

use crate::config::{metadata, DEFAULT_WORKSPACE};
use crate::params::SharedParams;
use crate::CliConfiguration;

use log::info;

#[derive(Debug, StructOpt)]
pub struct WorkspaceCmd {
	#[allow(missing_docs)]
	#[structopt(flatten)]
	pub shared_params: SharedParams,

	#[structopt(subcommand)]
	pub subcommand: WorkspaceSubCmd,
}
#[derive(Debug, StructOpt)]
pub enum WorkspaceSubCmd {
	/// List all workspace.
	List,
	/// Set current default workspace, could set a workspace which not in workspace list,
	Default(DefaultCmd),
	/// Delete a workspace, if the workspace has data, it would **remove all data** in this workspace.
	Delete(DeleteCmd),
}
#[derive(Debug, StructOpt)]
pub struct DefaultCmd {
	#[structopt(value_name = "DEFAULT WORKSPACE")]
	default: String,
}

#[derive(Debug, StructOpt)]
pub struct DeleteCmd {
	#[structopt(value_name = "DEL WORKSPACE")]
	deleted: String,
}
impl CliConfiguration for WorkspaceCmd {
	fn shared_params(&self) -> &SharedParams {
		&self.shared_params
	}
}
impl WorkspaceCmd {
	/// Run the workspace command, this function could run directly, do not need be wrapped by a runner.
	pub fn init_and_run<C: SubstrateCli>(&self) -> sc_cli::Result<()> {
		self.init::<C>()?;

		let base_path = self
			.base_path()?
			.unwrap_or_else(|| BasePath::from_project("", "", &C::executable_name()));

		metadata(&base_path, |mut m| {
			info!(
				"{}",
				Style::new()
					.bold()
					.fg(Color::Yellow)
					.paint("Current default workspace:")
			);
			let default = match m.current_workspace {
				Some(ref default) => {
					info!("	{}", default);
					default.clone()
				}
				None => {
					info!("	[Notice:have not set default workspace, would use \"{}\" as default workspace name]", DEFAULT_WORKSPACE);
					DEFAULT_WORKSPACE.to_string()
				}
			};
			info!(""); // add an empty line
			match self.subcommand {
				WorkspaceSubCmd::List => {
					info!(
						"{}",
						Style::new()
							.bold()
							.fg(Color::Yellow)
							.paint("List all recorded workspaces:")
					);
					match m.workspaces {
						Some(ref list) => {
							for i in list.iter() {
								let end = if i == &default {
									"	<---[default workspace]"
								} else {
									""
								};
								info!("	{}{}", i, end);
							}
						}
						None => info!("	current do not have any workspace!"),
					}
				}
				WorkspaceSubCmd::Default(ref cmd) => {
					info!(
						"{}",
						Style::new()
							.bold()
							.fg(Color::Yellow)
							.paint(format!("Set [{}] as default workspace.", cmd.default))
					);
					m.current_workspace = Some(cmd.default.clone());
				}
				WorkspaceSubCmd::Delete(ref cmd) => {
					info!(
						"{}",
						Style::new()
							.bold()
							.fg(Color::Yellow)
							.paint(format!("Delete workspace [{}].", cmd.deleted))
					);
					if let Some(ref default) = m.current_workspace {
						if default == &cmd.deleted {
							info!("	delete default record: [{}]", default);
							m.current_workspace = None;
						}
					}
					if let Some(ref mut list) = m.workspaces {
						// remove this workspace from workspace list
						list.retain(|item| {
							let r = item != &cmd.deleted;
							if !r {
								info!("	delete workspace:[{}] from workspace list", cmd.deleted);
							}
							r
						});
					}
					// remove all workspace data
					let p = base_path.path().join(&cmd.deleted);
					let _ = ::std::fs::remove_dir_all(p);
				}
			}
			m
		})?;
		Ok(())
	}
}
