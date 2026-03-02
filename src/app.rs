// --- constants ---
pub const DEFAULT_APP: &str = include_str!("../res/app.toml");
pub const DEFAULT_APP_CLEAN: &str = include_str!("../res/app_clean.toml");

// --- imports ---
use anyhow::{anyhow, bail, Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use indexmap::IndexMap as Map;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{table, value, Array, DocumentMut, Item, Table, Value};
use walkdir::WalkDir;
use crate::util::table::*;

// --- functions ---
pub fn sanitize_app_name<S: Into<String>>(name: S) -> String {
	name.into().trim().replace(' ', "_").replace('\\', "/").trim_matches('/').to_string()
}

pub fn new_app(path: &Path, name: String, clean: bool) -> Result<PathBuf> {
	let app_dir = path.join("apps");
	if !app_dir.exists() {
		fs::create_dir_all(&app_dir)?;
	}
	let path = app_dir.join(format!("{}.toml", sanitize_app_name(name)));
	if path.exists() {
		bail!(
			"file already exists: {}",
			path.display()
		);
	}
	fs::write(&path, match clean {
		true => DEFAULT_APP_CLEAN,
		false => DEFAULT_APP,
	})?;
	Ok(path)
}

// --- definitions ---
#[derive(Default, Deserialize, Serialize)]
pub struct App {
	pub meta: Option<Meta>,
	pub vars: Option<Map<String, String>>,
	pub env: Option<Map<String, String>>,
	pub cmds: Map<String, Cmd>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Meta {
	pub name: Option<String>,
	pub description: Option<String>,
	pub version: Option<String>,
}

#[derive(Default, Deserialize, Serialize)]
pub struct Cmd {
	pub bin: String,
	pub args: Vec<String>,
	pub env: Option<Map<String, String>>,
}

// --- implementations ---
impl Display for App {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let mut sections: Map<String, Map<String, String>> = Map::new();

		// 1. metadata
		if let Some(meta) = &self.meta {
			let mut meta_map = Map::new();

			let name = meta.name.as_deref().unwrap_or("Unspecified");
			let version = meta.version.as_deref().unwrap_or("Unspecified");

			meta_map.insert("Name".bright_yellow().to_string(), name.into());
			meta_map.insert("Version".bright_yellow().to_string(), version.into());
			if let Some(desc) = &meta.description {
				meta_map.insert("Description".bright_yellow().to_string(), desc.into());
			}

			sections.insert(format!("{}", "Metadata".bright_yellow().bold()), meta_map);
		}

		// 2. local vars
		let mut vars_map = Map::new();
		if let Some(vars) = &self.vars {
			if vars.is_empty() {
				vars_map.insert("(no custom variables provided)".bright_red().to_string(), "".into());
			}
			for (name, value) in vars {
				vars_map.insert(format!("${name}").bright_red().to_string(), value.clone());
			}
		}
		if !vars_map.is_empty() {
			sections.insert(format!("{}", "Local Variables".bright_red().bold()), vars_map);
		}

		// 3. local environment
		let mut env_map = Map::new();
		if let Some(env) = &self.env {
			if env.is_empty() {
				env_map.insert("(no environment overrides provided)".bright_blue().to_string(), "".into());
			}
			for (name, value) in env {
				env_map.insert(format!("${name}").bright_blue().to_string(), value.clone());
			}
		}
		if !env_map.is_empty() {
			sections.insert(format!("{}", "Local Environment".bright_blue().bold()), env_map);
		}

		// 5. generate main table
		let rows = generate_rows(sections);
		make_table(f, "App Info", rows)?;

		// 6. commands
		let mut cmd_sections: Map<String, Map<String, String>> = Map::new();

		for (name, cmd) in &self.cmds {
			let mut cmd_map = Map::new();
			cmd_map.insert(
				"Executable".bright_green().to_string(),
				cmd.bin.clone()
			);
			cmd_map.insert(
				"Arguments".bright_green().to_string(),
				shell_words::join(&cmd.args)
			);
			cmd_sections.insert(format!("{}", name.bright_green().bold()), cmd_map);

			let mut env_map = Map::new();
			if let Some(env) = &cmd.env {
				if env.is_empty() {
					env_map.insert("(no environment overrides provided)".bright_blue().to_string(), "".into());
				}
				for (name, value) in env {
					env_map.insert(format!("${name}").bright_blue().to_string(), value.clone());
				}
			}
			if !env_map.is_empty() {
				cmd_sections.insert(format!("{name}/Environment").bright_blue().bold().to_string(), env_map);
			}
		}


		// 5. generate commands table
		let rows = generate_rows(cmd_sections);
		make_table(f, "Commands", rows)?;

		Ok(())
	}
}

impl App {
	/// saves app into a definition file with proper formatting
	pub fn save(&self, app_file: &Path) -> Result<()> {
		// ensure the directory exists
		if let Some(parent) = app_file.parent() {
			fs::create_dir_all(parent)?;
		}

		// read the existing file or start with empty doc
		let mut doc = if app_file.exists() {
			let text = fs::read_to_string(app_file)
				.with_context(|| format!("failed to read app at '{}'", app_file.display()))?;
			text.parse::<DocumentMut>().context("failed to parse app toml")?
		} else {
			DocumentMut::new()
		};

		// 1. meta
		if let Some(meta) = &self.meta {
			if !doc.as_table().contains_key("meta") {
				doc["meta"] = table();
			}
			let table = doc["meta"].as_table_mut().unwrap();
			table.clear();

			if let Some(name) = &meta.name { table["name"] = value(name.to_string()); }
			if let Some(desc) = &meta.description { table["description"] = value(desc.to_string()); }
			if let Some(ver) = &meta.version { table["version"] = value(ver.to_string()); }
		} else {
			doc.as_table_mut().remove("meta");
		}

		// 2. vars
		if let Some(vars) = &self.vars {
			if !doc.as_table().contains_key("vars") {
				doc["vars"] = table();
			}
			let table = doc["vars"].as_table_mut().unwrap();
			table.clear();
			for (k, v) in vars { table[k] = value(v.clone()); }
		} else {
			doc.as_table_mut().remove("vars");
		}

		// 3. env
		if let Some(env) = &self.env {
			if !doc.as_table().contains_key("env") {
				doc["env"] = table();
			}
			let table = doc["env"].as_table_mut().unwrap();
			table.clear();
			for (k, v) in env { table[k] = value(v.clone()); }
		} else {
			doc.as_table_mut().remove("env");
		}

		// 4. cmds
		if !self.cmds.is_empty() {
			if !doc.as_table().contains_key("cmds") {
				doc["cmds"] = table();
			}
			let table = doc["cmds"].as_table_mut().unwrap();
			table.clear();

			for (cmd_name, cmd) in &self.cmds {
				let mut cmd_table = Table::new();
				cmd_table["bin"] = value(cmd.bin.clone());
				if !cmd.args.is_empty() {
					let mut arr = Array::new();
					for item in &cmd.args {
						arr.push(item.clone());
					}
					cmd_table["args"] = Item::Value(Value::Array(arr));
				}
				if let Some(env) = &cmd.env {
					let mut env_table = Table::new();
					for (k, v) in env { env_table[k] = value(v.clone()); }
					cmd_table["env"] = Item::Table(env_table);
				}
				table[cmd_name] = Item::Table(cmd_table);
			}
		} else {
			doc.as_table_mut().remove("cmds");
		}

		// 5. write back
		fs::write(app_file, doc.to_string())
			.with_context(|| format!("failed to write app to {:?}", app_file.display()))?;

		Ok(())
	}

	/// finds all app definitions in {config_path}/apps and returns a map of app name -> path to definition
	pub fn find_all(config_path: &Path) -> Map<String, PathBuf> {
		let mut apps = Map::new();
		let apps_dir = config_path.join("apps");

		if !apps_dir.exists() {
			return apps;
		}

		// walk through the apps directory
		for entry in WalkDir::new(apps_dir)
			.min_depth(1) // don't include the apps folder itself
			.into_iter()
			.filter_map(|e| e.ok())
		{
			let path = entry.path();

			// only care about .toml files
			if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {

				// sanitize the name/key
				// we want the path relative to the "apps" folder, without the .toml
				// e.g., "apps/games/doom.toml" -> "games/doom"
				if let Ok(relative_path) = path.strip_prefix(&config_path.join("apps")) {
					let mut name = relative_path.to_string_lossy().to_string();

					// remove .toml extension
					if name.ends_with(".toml") {
						name.truncate(name.len() - 5);
					}

					// normalize slashes and trim
					let sanitized_name = name
						.replace('\\', "/") // ensure cross-platform consistency
						.trim_matches('/')
						.to_string();

					apps.insert(sanitized_name, path.to_path_buf());
				}
			}
		}
		apps
	}

	// getters and setters

	pub fn get(&self, query: &str) -> Option<String> {
		let parts: Vec<&str> = query.split('.').collect();
		self.get_slice(parts.as_slice())
	}

	pub fn set(&mut self, query: &str, value: String) -> Result<()> {
		let parts: Vec<&str> = query.split('.').collect();
		self.set_slice(parts.as_slice(), value)
	}

	pub fn unset(&mut self, query: &str) -> Result<()> {
		let parts: Vec<&str> = query.split('.').collect();
		self.unset_slice(parts.as_slice())
	}

	pub fn get_slice(
		&self,
		parts: &[&str]
	) -> Option<String> {
		match parts {
			["*"] => {
				return Some(format!("{}", self));
			}
			["cmds", cmd, rest @ ..] => {
				let cmd = self.cmds.get(*cmd)?;

				match rest {
					["bin"] => Some(cmd.bin.clone()),
					["env", k] => cmd.env.as_ref()?.get(*k).cloned(),
					["args", num] => match *num {
						"*" => Some(
							shell_words::join(&cmd.args)
						),
						_ => {
							let index = num.parse::<usize>().ok()?;
							cmd.args.get(index).cloned()
						}
					}
					_ => None,
				}
			}
			["meta", field] => {
				let meta = self.meta.as_ref()?;
				match *field {
					"name" => meta.name.clone(),
					"description" => meta.description.clone(),
					"version" => meta.version.clone(),
					_ => None,
				}
			}
			["vars", k] => self.vars.as_ref()?.get(*k).cloned(),
			["env", k] => self.env.as_ref()?.get(*k).cloned(),
			_ => None,
		}
	}

	pub fn set_slice(
		&mut self,
		parts: &[&str],
		value: String
	) -> Result<()> {
		match parts {
			["cmds", cmd_name, rest @ ..] => {
				let cmd = self.cmds
					.get_mut(*cmd_name)
					.ok_or_else(|| anyhow!("command '{}' not found", cmd_name))?;

				match rest {
					["bin"] => {
						cmd.bin = value;
						Ok(())
					}
					["env", k] => {
						let env = cmd.env.get_or_insert_default();
						env.insert((*k).to_string(), value);
						Ok(())
					}
					["args", num] => match *num {
						"*" => {
							cmd.args = shell_words::split(&value)
								.map_err(|e| anyhow!("failed to parse args: {}", e))?;
							Ok(())
						}
						_ => {
							let index = num.parse::<usize>()
								.map_err(|_| anyhow!("invalid arg index '{}'", num))?;

							let arg = cmd.args
								.get_mut(index)
								.ok_or_else(|| anyhow!("arg index {} out of bounds", index))?;

							*arg = value;
							Ok(())
						}
					}
					_ => Err(anyhow!("invalid cmds path")),
				}
			}
			["meta", field] => {
				let meta = self.meta.get_or_insert_default();

				match *field {
					"name" => meta.name = Some(value),
					"description" => meta.description = Some(value),
					"version" => meta.version = Some(value),
					_ => return Err(anyhow!("invalid meta field '{}'", field)),
				}

				Ok(())
			}
			["vars", k] => {
				let vars = self.vars.get_or_insert_default();
				vars.insert((*k).to_string(), value);
				Ok(())
			}
			["env", k] => {
				let env = self.env.get_or_insert_default();
				env.insert((*k).to_string(), value);
				Ok(())
			}
			_ => Err(anyhow!("invalid path")),
		}
	}

	pub fn unset_slice(
		&mut self,
		parts: &[&str],
	) -> Result<()> {
		match parts {
			["*"] => {
				*self = Default::default();
				Ok(())
			}
			["cmds", cmd_name, rest @ ..] => {
				let cmd = self.cmds
					.get_mut(*cmd_name)
					.ok_or_else(|| anyhow!("command '{}' not found", cmd_name))?;

				match rest {
					["bin"] => {
						cmd.bin.clear();
						Ok(())
					}
					["env", k] => {
						match *k {
							"*" => {
								cmd.env = None;
							}
							_ => {
								let env = cmd.env
									.as_mut()
									.ok_or_else(|| anyhow!("env not initialized"))?;
								env.swap_remove(*k);
							}
						}
						Ok(())
					}
					["args", num] => match *num {
						"*" => {
							cmd.args.clear();
							Ok(())
						}
						_ => {
							let index = num.parse::<usize>()
								.map_err(|_| anyhow!("invalid arg index '{}'", num))?;

							if index >= cmd.args.len() {
								return Err(anyhow!("arg index {} out of bounds", index));
							}

							cmd.args.swap_remove(index);
							Ok(())
						}
					}
					_ => Err(anyhow!("invalid cmds path")),
				}
			}
			["meta", field] => {
				let meta = self.meta
					.as_mut()
					.ok_or_else(|| anyhow!("meta not initialized"))?;

				match *field {
					"*" => {
						self.meta = None;
					}
					"name" => meta.name = None,
					"description" => meta.description = None,
					"version" => meta.version = None,
					_ => return Err(anyhow!("invalid meta field '{}'", field)),
				}

				Ok(())
			}
			["vars", k] => {
				match *k {
					"*" => {
						self.vars = None;
					}
					_ => {
						let vars = self.vars
							.as_mut()
							.ok_or_else(|| anyhow!("vars not initialized"))?;
						vars.swap_remove(*k);
					}
				}
				Ok(())
			}
			["env", k] => {
				match *k {
					"*" => {
						self.env = None;
					}
					_ => {
						let env = self.env
							.as_mut()
							.ok_or_else(|| anyhow!("env not initialized"))?;
						env.swap_remove(*k);
					}
				}
				Ok(())
			}
			_ => Err(anyhow!("invalid path")),
		}
	}
}
