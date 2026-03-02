// --- imports ---
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// --- definitions ---
/// ran - run anything now
///
/// a simple but customizable command-line launcher for games and programs.
///
/// copyright (c) 2026 Hasibix Hasi.
/// licensed under apache 2.0.
#[derive(Parser)]
#[command(
	author,
	version,
	about,
	long_about,
	arg_required_else_help = true
)]
pub struct Cli {
	#[arg(
		long,
		env = "RANCFG",
		help = "path for config files (defaults to your platform's app data directory)/ran",
		long_help = "path for config files (e.g. general config or app list). defaults to your platform's app data directory under 'ran'.",
	)]
	pub config: Option<PathBuf>,

	#[command(subcommand)]
	pub cmd: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
	/// launches an app with the 'launch' command
	Launch {
		/// app to be launched
		name: String,
		/// arguments passed to the app
		args: Vec<String>,
		/// run the command in the background
		#[arg(short, long)]
		background: bool,
	},

	/// launches a specific command of an app
	Cmd {
		/// command to run
		cmd: String,
		/// app to be launched
		name: String,
		/// arguments passed to the command
		args: Vec<String>,
		/// run the command in the background
		#[arg(short, long)]
		background: bool,
	},

	/// application management subcommands
	#[command(subcommand)]
	App(AppCmd),

	/// global configuration management
	#[command(subcommand)]
	Config(ConfigCmd),

	/// app alias management
	#[command(subcommand)]
	Alias(AliasCmd),

	/// global variables management
	#[command(subcommand)]
	Var(VarCmd),
}

#[derive(Subcommand)]
pub enum AppCmd {
	/// lists all apps (defined in config_path/apps/)
	#[command(alias = "ls")]
	List,

	/// opens an app's definition file in your preferred text editor
	Edit {
		app: String,
	},

	/// prints all information about an app
	#[command(alias = "info")]
	Print {
		app: String,
		#[arg(short, long)]
		raw: bool,
	},

	/// gets a key's value from an app's definition
	Get {
		app: String,
		key: Option<String>,
		#[arg(short, long)]
		raw: bool,
	},

	/// sets a key's value in an app's definition
	Set {
		app: String,
		key: String,
		value: String,
	},

	/// unsets a key in an app's definition
	Unset {
		app: String,
		key: String,
	},

	/// creates a dummy app definition file
	#[command(alias = "new")]
	Create {
		app: String,
		/// exclude comments and unnecessary data
		#[arg(short, long)]
		clean: bool,
		/// automatically opens the created file in your text editor
		#[arg(short, long)]
		edit: bool,
	},

	/// deletes an app's definition file (toml only)
	#[command(alias = "rm")]
	#[command(alias = "remove")]
	Delete {
		app: String,
		/// skip confirmation prompts
		#[arg(short, long)]
		yes: bool,
	},
}

/// global configuration management
#[derive(Subcommand)]
pub enum ConfigCmd {
	/// prints the current config path
	Path,

	/// opens the global config file in your preferred text editor
	Edit,

	/// prints the current config
	#[command(alias = "info")]
	Print {
		/// print raw toml
		#[arg(short, long)]
		raw: bool,
	},

	/// gets a key from the config
	Get {
		key: Option<String>,
		/// print raw toml when key is not specified
		#[arg(short, long)]
		raw: bool,
	},

	/// sets a key in the config
	Set {
		key: String,
		value: String,
	},

	/// unsets a key in the config
	Unset {
		key: String,
	},

	/// generates or regenerates a default config file
	Init {
		/// skip confirmation prompts
		#[arg(short, long)]
		yes: bool,
		/// exclude comments and unnecessary data
		#[arg(short, long)]
		clean: bool,
		/// automatically opens the created file in your text editor
		#[arg(short, long)]
		edit: bool,
	},
}

/// alias management
#[derive(Subcommand)]
pub enum AliasCmd {
	/// lists all app aliases's resolved alias chain
	#[command(alias = "ls")]
	List,

	/// returns the resolved alias chain for an app alias, or just the imediate value for it
	Get {
		key: String,
		// don't resolve alias chain
		#[arg(short, long)]
		unresolved: bool,
	},

	/// sets an alias
	Set {
		key: String,
		value: String,
	},

	/// unsets an alias
	Unset {
		key: String,
	},
}

/// global variable management
#[derive(Subcommand)]
pub enum VarCmd {
	/// lists all custom variables
	#[command(alias = "ls")]
	List,

	/// gets a variable's value
	Get {
		key: String,
	},

	/// sets a variable
	Set {
		key: String,
		value: String,
	},

	/// unsets a variable
	Unset {
		key: String,
	},
}
