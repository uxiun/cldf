use core::borrow;
use std::{
	fmt::{Debug, Display},
	fs::File,
	path::Path,
};

use csv::{StringRecord, StringRecordIter};
use serde::Deserialize;

const PATH_PREFIX: &str = "cldf/";

pub fn read_csv<Row: TryFrom<StringRecord>, P: AsRef<Path> + Display>(
	path: P,
) -> Result<Vec<Row>, String>
where
	<Row as TryFrom<StringRecord>>::Error: Debug,
{
	let f = File::open(format!("{}{}", PATH_PREFIX, path)).map_err(|e| e.to_string())?;

	let mut rdr = csv::Reader::from_reader(f);
	let mut rows: Vec<Row> = vec![];
	// let mut broken = None ;
	for row in rdr.records() {
		match row {
			Err(e) => {
				println!("{e}");
				// broken = Some(e);
				break;
			}
			Ok(o) => match o.try_into() {
				Ok(ok) => rows.push(ok),
				Err(e) => {
					println!("{:?}", e);
					break;
				}
			},
		}
	}

	Ok(rows)
}

#[derive(Debug, Clone, Deserialize)]
pub struct Parameter {
	pub id: String,
	pub name: String,
	pub description: String,
	pub column_spec: String,
	pub chapter_id: usize,
}

#[derive(Debug, Clone)]
pub struct Value {
	pub id: String,
	pub language_id: String,
	pub parameter_id: String,
	pub value: usize,
	pub code_id: String,
	pub comment: String,
	pub source: String,
	pub example_id: String,
}

#[derive(Debug, Clone)]
pub struct Language {
	pub id: String,
	pub name: String,
	pub macroarea: String,
	pub latitude: f32,
	pub longitude: f32,
	pub glottocode: String,
	pub iso6393p3code: String,
	pub family: String,
	pub subfamily: String,
	pub genus: String,
	pub genus_icon: String,
	pub iso_codes: String,
	pub samples_100: bool,
	pub samples_200: bool,
	pub country_id: String,
	pub source: String,
	pub parent_id: String,
}

impl Language {
	pub fn columns() -> [&'static str; 17] {[
		"id",
	"name",
	"macroarea",
	"latitude",
	"longitude",
	"glottocode",
	"iso6393p3code",
	"family",
	"subfamily",
	"genus",
	"genus_icon",
	"iso_codes",
	"samples_100",
	"samples_200",
	"country_id",
	"source",
	"parent_id",
	]}
}

#[derive(Debug)]
pub struct Chapter {
	pub id: String,
	pub name: String,
	pub description: String,
	pub contributor: String,
	pub citation: String,
	pub wp_slug: String,
	pub number: usize,
	pub area_id: Option<usize>,
	pub source: String,
	pub contributor_id: String,
	pub with_contributor_id: String,
}

impl TryFrom<StringRecord> for Chapter {
	type Error = String;
	fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
		match value.into_iter().collect::<Vec<_>>().as_slice() {
			&[id, name, description, contributor, citation, wp_slug, number, area_id, source, contributor_id, with_contributor_id] => {
				match (number.parse::<usize>(), area_id.parse::<usize>()) {
					(Ok(number), area_id) => Ok(Self {
						id: id.to_string(),
						name: name.to_string(),
						description: description.to_string(),
						contributor: contributor.to_string(),
						citation: citation.to_string(),
						wp_slug: wp_slug.to_string(),
						number,
						area_id: area_id.ok(),
						source: source.to_string(),
						contributor_id: contributor_id.to_string(),
						with_contributor_id: with_contributor_id.to_string(),
					}),
					(Err(e), _) => Err(e.to_string()),
				}
			}
			x => Err(format!("{:?} does not match column number", x)),
		}
	}
}

impl TryFrom<StringRecord> for Language {
	type Error = String;
	fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
		match value.into_iter().collect::<Vec<_>>().as_slice() {
			&[id, name, macroarea, latitude, longitude, glottocode, iso6393p3code, family, subfamily, genus, genus_icon, iso_codes, samples_100, samples_200, country_id, source, parent_id] => {
				match (
					latitude.parse::<f32>(),
					longitude.parse::<f32>(),
					samples_100.parse::<bool>(),
					samples_200.parse::<bool>(),
				) {
					(Ok(latitude), Ok(longitude), Ok(samples_100), Ok(samples_200)) => Ok(Self {
						id: id.to_string(),
						name: name.to_string(),
						macroarea: macroarea.to_string(),
						latitude,
						longitude,
						glottocode: glottocode.to_string(),
						iso6393p3code: iso6393p3code.to_string(),
						family: family.to_string(),
						subfamily: subfamily.to_string(),
						genus: genus.to_string(),
						genus_icon: genus_icon.to_string(),
						iso_codes: iso_codes.to_string(),
						samples_100,
						samples_200,
						country_id: country_id.to_string(),
						source: source.to_string(),
						parent_id: parent_id.to_string(),
					}),
					(Err(e), _, _, _) => {
						println!("{e}");
						Err(e.to_string())
					}
					(Ok(_), Err(e), _, _) => {
						println!("{e}");
						Err(e.to_string())
					}
					(_, _, Err(e), _) => {
						println!("{e}");
						Err(e.to_string())
					}
					(_, _, _, Err(e)) => {
						println!("{e}");
						Err(e.to_string())
					}
				}
			}
			x => Err(format!("{:?} does not match column number", x)),
		}
	}
}

impl TryFrom<StringRecord> for Parameter {
	type Error = String;
	fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
		match value.into_iter().collect::<Vec<_>>().as_slice() {
			&[id, name, description, column_spec, chapter_id] => usize::from_str_radix(chapter_id, 10)
				.map(|ch| Self {
					id: id.to_string(),
					name: name.to_string(),
					description: description.to_string(),
					column_spec: column_spec.to_string(),
					chapter_id: ch,
				})
				.map_err(|e| e.to_string()),
			x => Err(format!("{:?} does not match column number", x)),
		}
	}
}

impl TryFrom<StringRecord> for Value {
	type Error = String;
	fn try_from(value: StringRecord) -> Result<Self, Self::Error> {
		match value.into_iter().collect::<Vec<_>>().as_slice() {
			&[id, language_id, parameter_id, value, code_id, comment, source, example_id] => {
				usize::from_str_radix(&value, 10)
					.map(|value| Self {
						id: id.to_string(),
						language_id: language_id.to_string(),
						parameter_id: parameter_id.to_string(),
						value,
						code_id: code_id.to_string(),
						comment: comment.to_string(),
						source: source.to_string(),
						example_id: example_id.to_string(),
					})
					.map_err(|e| e.to_string())
			}
			x => Err(format!("{:?} does not match column number", x)),
		}
	}
}

#[test]
fn read() {
	match read_csv::<Chapter, _>("chapters.csv") {
		Ok(rs) => {
			println!("{:#?}", rs);
		}

		Err(e) => {
			println!("{e}");
		}
	}
}

#[test]
fn read_lang() {
	match read_csv::<Language, _>("languages.csv") {
		Ok(rs) => {
			let fil = rs
				.into_iter()
				.filter(|l| l.name == "Japanese")
				.collect::<Vec<_>>();
			println!("{:#?}", fil);
		}

		Err(e) => {
			println!("{e}");
		}
	}
}
