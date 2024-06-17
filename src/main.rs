#![feature(
	let_chains,
	iter_intersperse,
	slice_group_by,
	exit_status_error,
	slice_flatten,
	const_trait_impl,
	result_option_inspect
)]
#![allow(dead_code)]

use collect::{
	collect_values_per_language, collect_values_per_param, get_my_languages, MyLanguage,
};

mod collect;
mod csvs;
mod graph;
mod util;

use clap::{Parser, Subcommand};
use graph::GraphLine;

#[derive(Debug, Parser)]
struct Cli {
	#[command(subcommand)]
	command: Subcommands,
}

#[derive(Debug, Subcommand)]
enum Subcommands {
	WriteCSV,
	GraphLine(GraphLine),
}

fn main() {
	let cli = Cli::parse();

	match cli.command {
		Subcommands::WriteCSV => {
			write_per_lang_csv();
		}

		Subcommands::GraphLine(a) => {
			a.another_plot().inspect_err(|e| println!("{e}"));
		}
	}
}

fn write_per_lang_csv() -> Result<(), String> {
	let collected_values_per_param = collect_values_per_param()?;
	let collected_values_per_lang = collect_values_per_language()?;
	let mylangs = get_my_languages(collected_values_per_param, collected_values_per_lang)?;

	for (lang_id, mylang) in mylangs {
		let path = format!("out/langs/{}.csv", lang_id);
		mylang.write_to_csv(path);
	}

	Ok(())
}
