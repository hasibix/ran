// --- imports ---
use console::measure_text_width;
use indexmap::IndexMap;
use std::fmt::{self, Formatter};
use terminal_size::{terminal_size, Height, Width};

// --- functions ---
/// wraps rows to fit terminal width
#[allow(unused)]
pub fn wrap_rows(rows: Vec<String>, term_w: usize) -> Vec<String> {
	let mut wrapped = Vec::new();

	for row in rows {
		let mut start = 0;
		let chars: Vec<char> = row.chars().collect(); // handle Unicode properly
		while start < measure_text_width(&row) {
			let end = (start + term_w).min(measure_text_width(&row));
			let slice: String = chars[start..end].iter().collect();
			wrapped.push(slice);
			start += term_w;
		}
	}

	wrapped
}

/// returns width of the current terminal, or 80 as a fallback if it can't be detected
pub fn get_term_width() -> usize {
	if let Some((Width(w), Height(_h))) = terminal_size() {
		w as usize
	} else {
		80 // fallback if we can't detect terminal
	}
}

/// creates a table with the given name and rows, fitting to terminal width.
pub fn make_table(f: &mut Formatter<'_>, name: &str, rows: Vec<String>) -> fmt::Result {
	let _term_w = get_term_width();
	// wrap rows to fit terminal width minus borders
	// let rows = wrap_rows(rows, term_w - 4);

	// find the longest wrapped row
	let mut longest_row = rows.iter().map(|s| measure_text_width(s) + 2).max().unwrap_or(0);

	// top border
	let mut top = format!("╭─ {} ", name);
	let toplen = measure_text_width(&top);
	if longest_row > toplen {
		let remaining = longest_row.saturating_sub(toplen);
		top.push_str(&"─".repeat(remaining));
	} else {
		longest_row = toplen
	}
	top.push_str("─╮");
	writeln!(f, "{top}")?;

	// content rows
	for row in rows {
		let mut line = format!("│ {}", row);
		let remaining = longest_row.saturating_sub(measure_text_width(&line));
		line.push_str(&" ".repeat(remaining));
		line.push_str(" │");
		writeln!(f, "{line}")?;
	}

	// bottom border
	let mut bottom = "╰─".to_string();
	let remaining = longest_row.saturating_sub(measure_text_width(&bottom));
	bottom.push_str(&"─".repeat(remaining));
	bottom.push_str("─╯");
	writeln!(f, "{bottom}")?;

	Ok(())
}

/// generates rows for the config/app info display, handling ANSI color codes and wrapping values to fit terminal width (though currently ANSI breaks on line-wraps).
/// expects sections in the format of section name → (key → value).
pub fn generate_rows(
	sections: IndexMap<String, IndexMap<String, String>>,
) -> Vec<String> {
	let term_w = get_term_width().saturating_sub(4); // borders handled in make_box

	// find longest key length
	let (longest_key_length, _) = sections
		.iter()
		.flat_map(|(_, h)| h.iter())
		.map(|(k, v)| (measure_text_width(k), measure_text_width(v)))
		.fold((0, 0), |(mk, mv), (k, v)| (mk.max(k), mv.max(v)));

	let key_col_width = longest_key_length;
	let value_start_col = key_col_width + 5; // " : "
	let value_max_width = term_w.saturating_sub(value_start_col).max(1);

	let mut rows = Vec::new();

	for (section_name, section) in sections {
		// section header
		rows.push(format!("[ {} ]", section_name));

		for (k, v) in section {
			let key_pad = " ".repeat(key_col_width - measure_text_width(&k));
			let indent = " ".repeat(value_start_col);

			// if value is empty, just print the key without " : "
			if v.is_empty() {
				rows.push(format!("  {}{}", k, key_pad));
				continue;
			}

			let words: Vec<&str> = v.split_whitespace().collect();
			let mut current_line = String::new();
			let mut first_line = true;

			for word in words {
				let word_len = measure_text_width(word);

				// word longer than line → hard split
				if word_len > value_max_width {
					if !current_line.is_empty() {
						let line = if first_line {
							format!("  {}{} : {}", k, key_pad, current_line)
						} else {
							format!("{}{}", indent, current_line)
						};
						rows.push(line.chars().take(term_w).collect());
						current_line.clear();
						first_line = false;
					}

					let chars: Vec<char> = word.chars().collect();
					let mut start = 0;
					while start < chars.len() {
						let end = (start + value_max_width).min(chars.len());
						let slice: String = chars[start..end].iter().collect();

						let line = if first_line {
							format!("  {}{} : {}", k, key_pad, slice)
						} else {
							format!("{}{}", indent, slice)
						};
						rows.push(line.chars().take(term_w).collect());

						start = end;
						first_line = false;
					}

					continue;
				}

				let new_len = if current_line.is_empty() {
					word_len
				} else {
					measure_text_width(&current_line) + 1 + word_len
				};

				if new_len > value_max_width {
					let line = if first_line {
						format!("  {}{} : {}", k, key_pad, current_line)
					} else {
						format!("{}{}", indent, current_line)
					};
					rows.push(line.chars().take(term_w).collect());

					current_line.clear();
					first_line = false;
				}

				if !current_line.is_empty() {
					current_line.push(' ');
				}
				current_line.push_str(word);
			}

			// flush remainder
			if !current_line.is_empty() {
				let line = if first_line {
					format!("  {}{} : {}", k, key_pad, current_line)
				} else {
					format!("{}{}", indent, current_line)
				};
				rows.push(line.chars().take(term_w).collect());
			}
		}

		// empty row between sections
		rows.push(String::new());
	}

	rows
}
