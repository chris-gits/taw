use clap::Parser;
use regex::Regex;

use std::path::PathBuf;

#[derive(Parser)]
#[clap(version, about, author)]
pub struct Arguments {
	// Walker
	#[arg(default_value = ".", help = "Path to be walked")]
	pub origin: PathBuf,
	#[arg(short, long, help = "Walk recursively")]
	pub recursive: bool,
	#[arg(short, long, help = "Canonicalize output paths")]
	pub canonicalize: bool,

	// Type Filter
	#[arg(short, long, group = "default_filter_redundance", help = "Only match files")]
	pub files: bool,
	#[arg(short, long, groups = ["default_filter_redundance", "directories_have_no_text"], help = "Only match directories")]
	pub directories: bool,

	// Regex Config
	#[arg(short, long, help = "Disable REGEX case-sensitivity")]
	pub ignore_case: bool,

	// Pattern Matches
	#[arg(short, long, help = "Match entries' name to REGEX pattern")]
	pub name: Option<Regex>,
	#[arg(short, long, groups = ["directories_have_no_text", "text_display_needs_newlines"], help = "Match entries' readable text to REGEX pattern")]
	pub text: Option<Regex>,

	// Display Options
	#[arg(short, long, group = "text_display_needs_newlines", help = "Display entries in a non-line-breaking format")]
	pub list: bool,
    
	// Debug Flags
	#[arg(short, long, help = "Enables debug warnings")]
	pub warnings: bool
}