use clap::Parser;
use regex::Regex;
use jwalk::WalkDir;
use colored::Colorize;

use std::path::PathBuf;
use std::fs::read;

#[derive(Parser)]
#[clap(version, about, author)]
struct Arguments {
	// Walker
	#[arg(default_value = ".", help = "Path to be walked")]
	origin: PathBuf,
	#[arg(short, long, help = "Walk recursively")]
	recursive: bool,
	#[arg(short, long, help = "Canonicalize output paths")]
	canonicalize: bool,
	// Type Filter
	#[arg(short, long, group = "default_filter_redundance", help = "Only match files")]
	files: bool,
	#[arg(short, long, groups = ["default_filter_redundance", "directories_have_no_text"], help = "Only match directories")]
	directories: bool,
	// Regex Config
	#[arg(short, long, help = "Disable REGEX case-sensitivity")]
	ignore_case: bool,
	// Pattern Matches
	#[arg(short, long, help = "Match entries' name to REGEX pattern")]
	name: Option<Regex>,
	#[arg(short, long, group = "directories_have_no_text", help = "Match entries' readable text to REGEX pattern")]
	text: Option<Regex>,
	// Debug Flags
	#[arg(short, long, help = "Enables debug warnings")]
	warnings: bool
}

fn main() {
	let mut args = Arguments::parse();

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
			warn!(format!("{}: {}", $message, $path.to_string_lossy()));
			continue;
		};
	}

	if !args.origin.exists() { fail!(format!("\"{}\" does not exist.", args.origin.to_string_lossy())); }

	if args.canonicalize {
		args.origin = match args.origin.canonicalize() {
			Err(_) => {
				fail!(format!("Could not canonicalize origin \"{}\"", args.origin.to_string_lossy()));
			}
			Ok(canon_path) => canon_path
		};
	}

	if args.ignore_case {
		if let Some(name_regex) = args.name {
			args.name = match Regex::new(format!("(?i){}", name_regex.as_str()).as_str()) {
				Err(_) => {fail!("Could not insert case-insensitivity flag to name pattern.");},
				Ok(modified_regex) => Some(modified_regex),
			}
		}
		if let Some(text_regex) = args.text {
			args.text = match Regex::new(format!("(?i){}", text_regex.as_str()).as_str()) {
				Err(_) => {fail!("Could not insert case-insensitivity flag to text pattern.");},
				Ok(modified_regex) => Some(modified_regex),
			}
		}
	}

	let mut walker = WalkDir::new(&args.origin).skip_hidden(false);
	if !args.recursive { walker = walker.max_depth(1) }

	for dir_entry_result in walker {
		if let Ok(dir_entry) = dir_entry_result {
			let entry_path = dir_entry.path();

			if args.origin.is_dir() && entry_path == args.origin { continue }
			if !(!args.directories && !args.files) {
				if entry_path.is_file() && !args.files { continue }
				if entry_path.is_dir() && !args.directories { continue }
			}

			// Matchers
			let result_path = match &args.name {
				None => entry_path.to_string_lossy().to_string(),
				Some(name_regex) => {
					let file_name = match entry_path.file_name() {
						None => {entry_warn!(entry_path, "Could not retrieve file name");},
						Some(file_name_osstr) => match file_name_osstr.to_str() {
							None => {entry_warn!(entry_path, "Could not interpret file name");},
							Some(file_name_str) => file_name_str
						}
					};
					let mut result_string = String::new();
					let mut first_flag = true;
					let mut last_index = 0;
					for capture in name_regex.captures_iter(file_name) {
						if first_flag {
							first_flag = false;
							if let Some(parent_path) = entry_path.parent() {
								result_string += &parent_path.to_string_lossy();
								result_string += "/";
							}
						}
						let first_capture = capture.get(0).unwrap();
						let start = first_capture.start();
						let end = first_capture.end();
						result_string += file_name.get(last_index..start).unwrap();
						result_string += &file_name.get(start..end).unwrap().green().bold().underline().to_string();
						last_index = end;
					}
					if first_flag { continue }
					else { result_string + file_name.get(last_index..).unwrap()}
				}
			};

			let result_lines = match &args.text {
				None => vec![],
				Some(text_regex) => {
					if entry_path.is_dir() { continue }
					let mut line_matches: Vec<String> = vec![];
					let mut line_matched_flag = false;
					for (line_index, line) in match String::from_utf8(
						match read(entry_path.clone()) {
							Err(_) => {entry_warn!(entry_path, "Could not open");},
							Ok(read_bytes) => read_bytes
						}
					) {
						Err(_) => {entry_warn!(entry_path, "Could not read");},
						Ok(read_string) => read_string
					}.to_string().lines().enumerate() {
						let mut result_string = String::new();
						let mut first_flag = true;
						let mut last_index = 0;
						for capture in text_regex.captures_iter(line) {
							if !line_matched_flag { line_matched_flag = true }
							if first_flag {
								first_flag = false;
								result_string += "\t";
								result_string += &(line_index + 1).to_string().bold().to_string();
								result_string += &": ".bold().to_string();
							}
							let first_capture = capture.get(0).unwrap();
							let start = first_capture.start();
							let end = first_capture.end();
							result_string += &line.get(last_index..start).unwrap().dimmed().italic().to_string();
							result_string += &line.get(start..end).unwrap().green().bold().underline().to_string();
							last_index = end;
						}
						if first_flag { continue }
						else { 
							line_matches.push(result_string + &line.get(last_index..).unwrap().dimmed().italic().to_string())
						}
					}
					if !line_matched_flag { continue }
					line_matches
				}
			};

			// Display results
			println!("{result_path}");
			if !result_lines.is_empty() { println!("{}", result_lines.join("\n")) }
		}
	}
}
