// --- imports ---
use anyhow::{anyhow, bail, Result};
use indexmap::IndexMap as Map;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use crate::app::App;
use crate::config::Config;
use crate::resolver::Resolver;
use crate::util::args::sandwich_args;

// --- definitions ---
pub struct Launcher {
	pub apps: Map<String, PathBuf>,
	pub config: Config,
}

// --- implementations ---
impl Launcher {
	/// interactively resolve app name conflicts
	pub fn conflict_resolver<'p>(
		&self,
		query: &str,
		matches: Vec<&'p Path>
	) -> Result<&'p Path> {
		// check if we are allowed to be interactive
		if self.config.noninteractive || !atty::is(atty::Stream::Stdout) {
			bail!(
				"multiple results for query '{query}': {}",
				matches
				.iter()
				.map(|&p| p.to_string_lossy().into_owned())
				.collect::<Vec<String>>()
				.join(", ")
			);
		}

		// interactive selection
		use dialoguer::{theme::ColorfulTheme, Select};

		let items: Vec<String> = matches
		.iter()
		.map(|p| p.to_string_lossy().to_string())
		.collect();

		let selection = Select::with_theme(&ColorfulTheme::default())
		.with_prompt("multiple apps found. please select one:")
		.items(&items)
		.default(0)
		.interact_opt()?;

		match selection {
			Some(index) => Ok(matches[index]),
			None => bail!("cancelled app selection."), // esc/ctrl+c
		}
	}

	/// (private) finds app from query with stack tracking
	fn find_app_inner(&self, query: &str, stack: Vec<String>) -> Result<&Path> {
		if stack.contains(&query.into()) {
			let mut stack = stack;
			stack.push(query.into());
			bail!("infinite recursion in alias expansion: {}", stack.join(" -> "));
		}
		let query = query.trim().trim_matches('/');
		if query.is_empty() { bail!("app definition not found for '{query}'") }

		if let Some(alias) = &self.config.alias {
			if let Some(app) = alias.get(query) {
				let mut stack = stack;
				stack.push(query.to_string());
				return self.find_app_inner(app, stack);
			}
		}

		let matches: Vec<&Path> = self.apps.iter()
		.filter(|(full_name, _)| {
			let leaf_name = full_name.split('/').last().unwrap_or(full_name);
			full_name == &query || leaf_name == query
		})
		.map(|(_, path)| path.as_path())
		.collect();

		if matches.len() > 0 {
			match matches.len() {
				1 => Ok(matches.get(0).ok_or(anyhow!("app definition not found for {query}"))?),
				_ => Ok(self.conflict_resolver(query, matches)?)
			}
		} else {
			bail!("app definition not found for {query}");
		}
	}

	/// finds app from query, resolving aliases, and errors on circular references
	pub fn find_app(&self, query: &str) -> Result<&Path> {
		self.find_app_inner(query, vec![])
	}

	/// loads app from query, resolving aliases, and errors on circular references
	pub fn load_app(&self, query: &str) -> Result<App> {
		let path = self.find_app(query)?;
		let content = fs::read_to_string(path)?;
		Ok(toml::from_str(&content)?)
	}

	/// loads app from path, without resolving aliases
	pub fn load_app_from(&self, path: &Path) -> Result<App> {
		let content = fs::read_to_string(path)?;
		Ok(toml::from_str(&content)?)
	}

	/// initializes launcher by scanning for apps and loading config
	pub fn init(config_path: &Path, config: Config) -> Result<Launcher> {
		let app_path = config_path.join("apps");
		if !app_path.exists() {
			fs::create_dir_all(app_path)?;
		}
		let apps = App::find_all(config_path);
		Ok(Launcher {
			apps,
			config,
		})
	}

	/// launch an app by query with a specified command, with cli args and env, resolving aliases, and errors on circular references
	pub fn launch_app(
		&self,
		cmd: &str,
		query: &str,
		args: Vec<String>,
		env: Map<String, String>,
		background: bool
	) -> Result<()> {
		let resolver = Resolver::new(self);

		// 1. resolve @chain
		let path = self.find_app(query)?;
		let name = self.apps.iter().find(|(_, p)| *p == path).map(|(n, _)| n).ok_or(anyhow!("app definition not found for {query}"))?;
		let app = self.load_app_from(path)?;
		let parts = resolver.resolve_command(&app, cmd)?;

		// 2. sandwich args (%! replacement)
		let intermediate_args = sandwich_args(parts.args, args);

		// 3. layer envs
		let mut final_env = env;
		if let Some(env) = &self.config.env {
			final_env.extend(env.clone());
		}
		final_env.extend(parts.env);

		// 4. resolve variable (only on what we are about to use)
		let final_bin = resolver.expand(Some(&app), &parts.bin)?;

		let final_args: Vec<String> = intermediate_args
			.into_iter()
			.map(|arg| resolver.expand(Some(&app), &arg))
			.collect::<Result<Vec<_>>>()?;

		let final_env: Map<String, String> = final_env
			.into_iter()
			.map(|(k, v)| {
				resolver.expand(Some(&app), &v)
					.map(|expanded| (k, expanded))
			})
			.collect::<Result<Map<_, _>>>()?;

		// 5. build and launch
		if background {
			let mut proc = Command::new(final_bin);
			proc.args(final_args)
				.stdin(Stdio::null())
				.stdout(Stdio::null())
				.stderr(Stdio::null())
				.current_dir(std::env::current_dir()?);
			// spawn and immediately forget
			let _ = proc.spawn();
			match cmd {
				"launch" => println!("launched app '{name}' in the background!"),
				_ => println!("started executing command '{cmd}' for app '{name}' in the background!"),
			}
		} else {
			let mut proc = Command::new(final_bin);
			proc.args(final_args).envs(final_env).current_dir(std::env::current_dir()?);
			// wait for exit
			match cmd {
				"launch" => println!("launching app '{name}'..."),
				_ => println!("running command '{cmd}' for app '{name}'..."),
			}
			let status = proc.status()?;
			if !status.success() {
				eprintln!("process exited with {}", status);
			}
		}
		Ok(())
	}
}
