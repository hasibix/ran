// --- imports ---
use anyhow::{anyhow, bail, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

// --- functions ---
/// returns a 'default' config path according to OS standards.
pub fn default_config_path(name: &str) -> Result<PathBuf> {
	let base = dirs::config_dir().ok_or(anyhow!("could not determine a config directory"))?;
	Ok(base.join(name))
}

/// opens 'path' in $VISUAL, $EDITOR, or OS-wide default program, or notepad/nano
pub fn open_in_editor(path: &Path, create: bool) -> Result<()> {
	if !path.exists() {
		if !create {
			bail!("file '{}' does not exist", path.display());
		}
		if let Some(parent) = path.parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(path, "")?;
	}

	// 1. try $VISUAL / $EDITOR
	if let Some(editor) = std::env::var("VISUAL").ok()
		.or_else(|| std::env::var("EDITOR").ok())
	{
		// try opening with the user’s editor
		if Command::new(&editor).arg(path).status().is_ok() {
			return Ok(());
		}
	}

	// 2. try system default opener (xdg-open / open / start)
	if open::that(path).is_ok() {
		return Ok(());
	}

	// 3. fallback terminal editors
	#[cfg(windows)]
	{
		Command::new("notepad").arg(path).status()?;
	}
	#[cfg(not(windows))]
	{
		Command::new("nano").arg(path).status()?;
	}

	Ok(())
}
