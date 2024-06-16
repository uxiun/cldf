#![feature(iter_intersperse, slice_group_by, exit_status_error, slice_flatten, const_trait_impl)]
#![allow(dead_code)]

use collect::{collect_values_per_language, collect_values_per_param, get_my_languages, MyLanguage};

mod csvs;
mod util;
mod collect;

fn main() {
	write_per_lang_csv();
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
