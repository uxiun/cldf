use std::{collections::{HashMap, HashSet}, fs::OpenOptions, io::{BufWriter, Write}, path::Path};

use crate::csvs::{read_csv, Chapter, Language, Parameter, Value};

pub struct MyLanguage {
	language: Language,
	param_values: HashMap<String, SameValue>
}

pub struct SameValue {
	value: usize,
	same_value_language_ids: Vec<String>,
	value_language_count: usize,
	language_count_total: usize,
	count_ratio: f32
}

impl SameValue {
	fn columns() -> [&'static str; 4] {
		[
			"value",
			"same_value_language_ids",
			"value_language_count",
			"language_count_total"
		]
	}
}

type ValueLanguagesMap = HashMap<usize, HashSet<String>>; // <value, language_ids>

impl MyLanguage {
	fn columns() -> [&'static str; 10] {
		[
			"parameter_id",
			"parameter_name",
			"parameter_description",
			"chapter_id",
			"chapter_citation",
			"value",
			"same_value_language_ids",
			"value_language_count",
			"language_count_total",
			"count_ratio"
		]
	}

	fn rows(self,
		parameters: HashMap<String, Parameter>,
		chapters: HashMap<usize, Chapter>
	) -> Vec<[String ; 10]> {
		self.param_values.into_iter()
		.filter_map(|(parameter_id, samevalue)| {
			let p = parameters.get(&parameter_id)?.clone();
			let ch = chapters.get(&p.chapter_id)?;

			let mut sames = samevalue.same_value_language_ids;
			sames.sort();

			Some([
				parameter_id,
				p.name.replace(",", ";"),
				p.description.replace(",", ";"),
				p.chapter_id.to_string(),
				ch.citation.replace(",", ";"),
				samevalue.value.to_string(),
				sames.into_iter()
					.intersperse(" ".to_string())
					.collect::<String>(),
				samevalue.value_language_count.to_string(),
				samevalue.language_count_total.to_string(),
				samevalue.count_ratio.to_string()
			])
		})
		.collect()
	}

	pub fn write_to_csv<P: AsRef<Path>>(self, path: P) -> Result<(), String> {
		let columns = Self::columns()
			.into_iter()
			.intersperse(",")
			.collect::<String>();

		let parameters =
			read_csv::<Parameter,_>("parameters.csv")?
			.into_iter()
			.map(|l| (l.id.clone(), l))
			.collect();

		let chapters =
			read_csv::<Chapter,_>("chapters.csv")?
			.into_iter()
			.filter_map(|l| l.id.parse::<usize>().map(|id| (id , l)).ok() )
			.collect();


		
		let rows = self.rows(parameters, chapters)
			.into_iter()
			.map(|row|
				row.into_iter()
				.intersperse(",".to_string())
				.collect::<String>()
			)
			.collect::<Vec<_>>();


		let mut lines = vec![columns];
		lines.extend(rows.into_iter());

		let f = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(path)
			.map_err(|e| e.to_string())?;

		let mut b = BufWriter::new(f);

		for line in lines {
			match b.write_all((line + "\n").as_bytes()) {
				Ok(_) => {},
				Err(e) => print!("{e}")
			}
		}

		Ok(())

	}
}

pub fn get_my_languages(
	collected_values_per_param: HashMap<String, ValueLanguagesMap>,
	collected_values_per_lang: HashMap<String, ParamValueMap>,
) -> Result<HashMap<String, MyLanguage>, String> {
	let langs: Vec<Language> = read_csv("languages.csv")?;

	Ok(
	langs.into_iter()
		.filter_map(|lang| {
			let language = lang.clone();
			let paramvalue = collected_values_per_lang.get(&lang.id)?;

			let param_values: HashMap<String, SameValue> =
				paramvalue.into_iter()
				.filter_map(|(param_id, value)| {
					let value_langs = collected_values_per_param.get(param_id)?;
					let total: usize = collected_values_per_param
						.iter()
						.map(|(_, valuelangs)| valuelangs.values().map(|s| s.len()).sum::<usize>() )
						.sum();
					let sames = value_langs.get(value)?;
					let count = sames.len();
					let sames = sames.into_iter()
						.filter(|lang_id| lang_id.to_string() != lang.id)
						.cloned()
						.collect::<Vec<_>>();

					let samevalue: SameValue = SameValue {
						value: *value,
						same_value_language_ids: sames,
						value_language_count: count,
						language_count_total: total,
						count_ratio: count as f32 / total as f32
					};

					Some((param_id.to_owned(), samevalue))
				})
				.collect();

			Some((lang.id, MyLanguage {
				language,
				param_values
			}))
		})
		.collect()
	)
}

pub fn collect_values_per_param() -> Result<HashMap<String, ValueLanguagesMap>, String> {
	let values: Vec<Value> = read_csv("values.csv")?;

	let mut rares: HashMap<String, ValueLanguagesMap> = HashMap::new();

	for value in values {
		rares.entry(value.parameter_id)
			.and_modify(|h| {
				h.entry(value.value)
				.and_modify(|set| {set.insert(value.language_id.clone()); })
				.or_insert(HashSet::from_iter([value.language_id.clone()])); }
			)
			.or_insert(HashMap::from_iter([
				(value.value, HashSet::from_iter([value.language_id]))
			])
			);
	}

	Ok(rares)
}

type ParamValueMap = HashMap<String, usize>;

pub fn collect_values_per_language() -> Result<HashMap<String, ParamValueMap>, String> {
	let values: Vec<Value> = read_csv("values.csv")?;

	let mut hm: HashMap<String, ParamValueMap> = HashMap::new();

	for value in values {
		hm.entry(value.language_id)
			.and_modify(|h| {
				h.entry(value.parameter_id.clone())
				.and_modify(|set| {
					*set = value.value; })
				.or_insert(value.value);
			})
			.or_insert(HashMap::from_iter([
				(value.parameter_id, value.value)
			])
			);
	}

	Ok(hm)
}
// }