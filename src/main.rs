// ran - run anything now
// a simple but customizable command-line launcher for games and programs.

// Copyright 2026 Hasibix Hasi

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//     http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// --- modules ---
mod util;
mod app;
mod cli;
mod config;
mod handler;
mod launcher;
mod resolver;

// --- imports ---
use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;
use crate::cli::*;
use crate::handler::CommandHandler;
use crate::util::fs::default_config_path;

// --- functions ---
fn main() {
	if let Some(e) = real_main().err() {
		eprintln!("{}", e);
	}
}

fn real_main() -> Result<()> {
	let cli = Cli::parse();
	let config_path = if let Some(c) = cli.config {
		PathBuf::from(c)
	} else {
		default_config_path("ran")?
	};
	let handler = CommandHandler::new(config_path);

	let cmd = cli.cmd.ok_or(anyhow!("no command was supplied"))?;
	handler.handle_command(cmd)
}
