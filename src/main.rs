// External Imports
use clap::Parser;
use regex::Regex;
use jwalk::WalkDir;
use colored::Colorize;

// Standard Imports
use std::fs::read;

// Internal Imports
mod args;

fn main() {
	// Args. Parse
	let mut args = args::Arguments::parse();

	// Internal macros
	macro_rules! fail {
		($message:expr) => {
			println!("{}", $message.red().bold());
			std::process::exit(1);
		};
	}
	macro_rules! warn {
		($message:expr) => {
			if args.warnings {
				println!("{}", $message.yellow());
			}
		};
	}
	macro_rules! entry_warn {
		($path:expr, $message:expr) => {
			warn!(format!("{} \"{}\"", $message, $path.to_string_lossy()));
			continue;
		};
	}

	// Validity checks
	if !args.origin.exists() { fail!(format!("\"{}\" does not exist", args.origin.to_string_lossy())); }

	// Args. Transformation
	if args.canonicalize {
		args.origin = match args.origin.canonicalize() {
			Err(_) => {
				fail!(format!("Could not canonicalize \"{}\"", args.origin.to_string_lossy()));
			}
			Ok(canon_path) => canon_path
		};
	}
	if args.ignore_case {
		if let Some(name_pattern) = args.name {
			args.name = match Regex::new(format!("(?i){}", name_pattern.as_str()).as_str()) {
				Err(_) => {fail!("Could not make name pattern case-insensitive");},
				Ok(modified_pattern) => Some(modified_pattern),
			}
		}
		if let Some(text_pattern) = args.text {
			args.text = match Regex::new(format!("(?i){}", text_pattern.as_str()).as_str()) {
				Err(_) => {fail!("Could not make text pattern case-insensitive");},
				Ok(modified_pattern) => Some(modified_pattern),
			}
		}
	}
	
	// Walker construction
	let mut walker = WalkDir::new(&args.origin).skip_hidden(false);
	if !args.recursive { walker = walker.max_depth(1) }

	// Entry walk
	let mut entries_list: Vec<String> = vec![];
	for dir_entry_result in walker {
		if let Ok(dir_entry) = dir_entry_result {
			let entry_path = dir_entry.path();

			// Type filters evaluation
			if args.origin.is_dir() && entry_path == args.origin { continue }
			if !(!args.directories && !args.files) {
				if entry_path.is_file() && !args.files { continue }
				if entry_path.is_dir() && !args.directories { continue }
			}

			// Name matcher evaluation
			let display_path = match &args.name {
				None => entry_path.to_string_lossy().to_string(),
				Some(name_pattern) => {
					// File name retrieval
					let entry_name = match entry_path.file_name() {
						None => {entry_warn!(entry_path, "Could not retrieve name");},
						Some(file_name_osstr) => match file_name_osstr.to_str() {
							None => {entry_warn!(entry_path, "Could not interpret name");},
							Some(file_name_str) => file_name_str
						}
					};
					
					// Captures iteration
					let mut display_path_buf = String::new();
					let mut first_capture = true;
					let mut last_index = 0;
					for capture in name_pattern.captures_iter(entry_name) {
						// Parent path push to display path buffer upon first iteration
						if first_capture {
							first_capture = false;
							if let Some(parent_path) = entry_path.parent() {
								display_path_buf += &parent_path.to_string_lossy();
								display_path_buf += "/";
							}
						}
						
						// Captured entry name push to display path buffer
						let first_capture = capture.get(0).unwrap();
						let start = first_capture.start();
						let end = first_capture.end();
						display_path_buf += entry_name.get(last_index..start).unwrap();
						display_path_buf += &entry_name.get(start..end).unwrap().green().bold().underline().to_string();
						last_index = end;
					}

					// Entry iterator continuation upon no captures found
					if first_capture { continue }
					// Display path buffer with remaining slice return 
					else {
						display_path_buf + entry_name.get(last_index..).unwrap()
					}
				}
			};

			// Text matcher evaluation
			let display_text_lines = match &args.text {
				None => vec![],
				Some(text_pattern) => {
					// Directory skip
					if entry_path.is_dir() { continue }

					// Entry text lines iteration
					let mut display_text_lines_buf: Vec<String> = vec![];
					let mut line_matched_flag = false;
					for (line_index, line) in
					match String::from_utf8(
						match read(entry_path.clone()) {
							Err(_) => {entry_warn!(entry_path, "Could not read");},
							Ok(read_bytes) => read_bytes
						}
					) {
						Err(_) => {entry_warn!(entry_path, "Could not decode");},
						Ok(read_string) => read_string
					}.to_string().lines().enumerate() {
						// Captures iteration
						let mut display_line_buf = String::new();
						let mut first_capture = true;
						let mut last_index = 0;
						for capture in text_pattern.captures_iter(line) {
							if !line_matched_flag { line_matched_flag = true }

							// Text push from start of line to start of capture to display line buffer upon first iteration
							if first_capture {
								first_capture = false;
								display_line_buf += "\t";
								display_line_buf += &(line_index + 1).to_string().bold().to_string();
								display_line_buf += &": ".bold().to_string();
							}

							// Captured text push to display line buffer
							let first_capture = capture.get(0).unwrap();
							let start = first_capture.start();
							let end = first_capture.end();
							display_line_buf += &line.get(last_index..start).unwrap().dimmed().italic().to_string();
							display_line_buf += &line.get(start..end).unwrap().green().bold().underline().to_string();
							last_index = end;
						}

						// Line iterator continuation upon no inner-line captures found
						if first_capture { continue }
						// Display line buffer with remaining slice push into display text lines buffer
						else { 
							display_text_lines_buf.push(display_line_buf + &line.get(last_index..).unwrap().dimmed().italic().to_string())
						}
					}

					// Entry iterator continuation upon no line captures found
					if !line_matched_flag { continue }

					display_text_lines_buf
				}
			};
			
			// Results display
			if args.list {
				entries_list.push(display_path);
			} else {
				println!("{display_path}");
			};
			if !display_text_lines.is_empty() {
				println!("{}", display_text_lines.join("\n"))
			}
		}
	}

	// Listed entries display 
	if entries_list.len() > 0 {
		println!("{}", entries_list.join(" "));
	}
}
