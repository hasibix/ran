// --- functions ---
/// puts child args in place of %! in parent args, or appends if no %! is found
pub fn sandwich_args(parent: Vec<String>, child: Vec<String>) -> Vec<String> {
	// find the index of the injection point
	if let Some(pos) = parent.iter().position(|arg| arg == "%!") {
		let mut final_args = Vec::new();

		// take everything BEFORE %!
		final_args.extend(parent[..pos].iter().cloned());

		// put the child args in the middle
		final_args.extend(child);

		// take everything AFTER %!
		final_args.extend(parent[pos + 1..].iter().cloned());

		final_args
	} else {
		// fallback if no %! is found: just append
		let mut fallback = parent;
		fallback.extend(child);
		fallback
	}
}

/// parse boolean from cli arg
pub fn parse_bool(s: &str) -> Option<bool> {
	match s.to_lowercase().trim() {
		"1" | "true" | "yes" | "y" | "on" => Some(true),
		"0" | "false" | "no" | "n" | "off" => Some(false),
		_ => None,
	}
}
