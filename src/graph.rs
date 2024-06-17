use std::{
	collections::HashMap,
	fs::OpenOptions,
	io::{BufWriter, Write},
	path::Path,
};

use clap::Args;
use gnuplot::{
	AutoOption, Axes2D, AxesCommon, Coordinate, DashType, Figure, LabelOption,
	PlotOption,
};

use crate::{
	collect::MyLanguage,
	csvs::{read_csv, Chapter, Parameter},
	util::{map_keys_dict, map_map_dict, rev_dict, transform_ddict, unzip_dict},
};

const POINT_SYMBOLS: &str = "+xtosdr";

#[cfg(test)]
mod tests {
	use gnuplot::{Figure, PlotOption};

	use crate::util::unzip_dict;

	use super::POINT_SYMBOLS;

	const J: [(i32, i32); 4] = [(1, 1), (2, 2), (3, 3), (4, 4)];

	const K: [(i32, i32); 4] = [(1, 1), (2, 1), (3, 1), (4, 1)];

	const L: [(i32, i32); 4] = [(1, 1), (2, 4), (3, 8), (4, 16)];

	#[test]
	fn legend() {
		let mut figure = Figure::new();
		let mut ax = figure.axes2d();
		let chars = POINT_SYMBOLS.chars().collect::<Vec<_>>();
		for (i, (label, xys)) in [("J", J), ("K", K), ("L", L)].into_iter().enumerate() {
			let (xs, ys) = unzip_dict(xys);
			ax.lines_points(
				xs,
				ys,
				&[
					PlotOption::Caption(label),
					PlotOption::PointSymbol(chars[i]),
				],
			);
		}

		figure
			.save_to_png("plot/test_legend.png", 640, 360)
			.inspect_err(|e| println!("{e}@save_to_png"));
	}
}

#[derive(Debug, Args)]
pub struct GraphLine {
	lang_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct GraphSaveOption {
	width_px: u32,
	height_px: u32,
}

impl GraphLine {
	pub fn sort_lang_ids(&mut self) {
		self.lang_ids.sort();
	}

	pub fn another_plot(self) -> Result<(), String> {
		let filename = self.filename_by_ids();
		let mylangs = MyLanguage::get_my_languages_by_id(&self.lang_ids)?;

		let mylangs = mylangs.values().collect::<Vec<_>>();

		let gs: AnotherGraphSource = mylangs.into();
		let gs = gs.sort_by_distinct_count_max_asc();

		let param_map = read_csv::<Parameter, _>("parameters.csv")?
			.into_iter()
			.map(|p| (p.id.clone(), p))
			.collect();

		let chapter_map = read_csv::<Chapter, _>("chapters.csv")?
			.into_iter()
			.filter_map(|p| p.id.parse::<usize>().map(|n| (n, p)).ok())
			.collect();

		gs.write_param_number_id_map(format!("plot/{}-param.txt", &filename), &param_map, &chapter_map)?;
		let mut figure = Figure::new();
		let ax = figure.axes2d();
		ax.set_x_ticks(Some((AutoOption::Fix(10.0), 1)), &[], &[]);
		ax.set_y_ticks(Some((AutoOption::Fix(5.0), 4)), &[], &[]);
		ax.set_grid_options(false, &[PlotOption::LineStyle(DashType::Solid)]);
		ax.set_minor_grid_options(&[PlotOption::LineStyle(DashType::SmallDot)]);
		ax.set_x_grid(true);
		ax.set_y_grid(true);
		ax.set_x_minor_grid(true);
		ax.set_y_minor_grid(true);
		gs.plot_param_number_id_map(ax, &param_map);
		gs.plot(ax);

		let op = GraphSaveOption {
			width_px: 2560,
			height_px: 360,
		};

		figure
			.save_to_svg(format!("plot/{}.svg", filename), op.width_px, op.height_px)
			.map_err(|e| format!("{e} @save_to_svg"))?;

		figure
			.save_to_png(format!("plot/{}.png", filename), op.width_px, op.height_px)
			.map_err(|e| format!("{e} @save_to_png"))?;

		Ok(())
	}

	// pub fn plot(self) -> Result<(), String> {
	// 	let filename = self.filename_by_ids();
	// 	let mylangs = MyLanguage::get_my_languages_by_id(&self.lang_ids)?;

	// 	let mylangs = mylangs.values().collect::<Vec<_>>();

	// 	let mut gs: GraphSource = mylangs.into();
	// 	gs.sort_by_distinct_count_max_asc();
	// 	let param_map = read_csv::<Parameter, _>("parameters.csv")?
	// 		.into_iter()
	// 		.map(|p| (p.id.clone(), p))
	// 		.collect();
	// 	gs.write_param_number_id_map(format!("plot/{}-param.txt", &filename), &param_map)?;
	// 	let mut figure = Figure::new();
	// 	let mut ax = figure.axes2d();
	// 	gs.plot_param_number_id_map(ax, &param_map);
	// 	gs.plot(ax);

	// 	let op = GraphSaveOption {
	// 		width_px: 2560,
	// 		height_px: 360,
	// 	};

	// 	figure
	// 		.save_to_svg(format!("plot/{}.svg", filename), op.width_px, op.height_px)
	// 		.map_err(|e| format!("{e} @save_to_svg"))?;

	// 	figure
	// 		.save_to_png(format!("plot/{}.png", filename), op.width_px, op.height_px)
	// 		.map_err(|e| format!("{e} @save_to_png"))?;

	// 	Ok(())
	// }

	fn filename_by_ids(&self) -> String {
		let mut ids = self .lang_ids .clone();
		ids.sort();
		ids.into_iter()
			.intersperse("_".to_string())
			.collect()
	}
}

#[derive(Debug, Clone)]
pub struct GraphSource {
	lang_params_map: HashMap<String, HashMap<usize, usize>>,
	param_number_id_map: HashMap<usize, String>,
}

#[derive(Debug, Clone)]
pub struct AnotherGraphSource {
	lang_params_map: HashMap<String, HashMap<String, usize>>,
	param_id_number_map: HashMap<String, usize>,
}

// impl GraphSource {
// 	fn sort_by_distinct_count_max_asc(&mut self) -> Self {
// 		// dbg!(&self.param_number_id_map);
// 		let paramid_valuelangs = map_keys_dict(
// 			transform_ddict(self.lang_params_map.clone()),
// 			&self.param_number_id_map,
// 		);
// 		// dbg!(&paramid_valuelangs);

// 		let mut ps = paramid_valuelangs
// 			.into_iter()
// 			.filter_map(|(param_id, h)| {
// 				let ma = h.keys().max()?;
// 				Some((param_id, h.len(), *ma))
// 			})
// 			.collect::<Vec<_>>();
// 		ps.sort_by_key(|(_, len, max)| len * 100 + max);
// 		// dbg!(&ps);

// 		let id_new_number: HashMap<String, usize> = ps
// 			.into_iter()
// 			.map(|(k, _, _)| k)
// 			.enumerate()
// 			.map(|(i, k)| (k, i))
// 			.collect();

// 		let old_new_number = map_map_dict(self.param_number_id_map.clone(), &id_new_number);
// 		// dbg!(&old_new_number);

// 		let lang_params_map = self
// 			.lang_params_map
// 			.clone()
// 			.into_iter()
// 			.map(|(lang_id, h)| {
// 				// let g = h.into_iter()
// 				// 	.filter_map(|(param_num, value)| old_new_number.get(&param_num).map(|n| (*n, *value)) )
// 				// 	.collect();

// 				let g = map_keys_dict(h, &old_new_number);

// 				(lang_id.to_owned(), g)
// 			})
// 			.collect();

// 		let param_number_id_map = rev_dict(id_new_number);

// 		Self {
// 			lang_params_map,
// 			param_number_id_map,
// 		}
// 	}

// 	fn write_param_number_id_map<P: AsRef<Path>>(
// 		&self,
// 		path: P,
// 		param_map: &HashMap<String, Parameter>,
// 	) -> Result<(), String> {
// 		let f = OpenOptions::new()
// 			.truncate(true)
// 			.create(true)
// 			.write(true)
// 			.open(path)
// 			.map_err(|e| format!("{e} @write_param_number_id_map"))?;
// 		let mut w = BufWriter::new(f);

// 		let mut v = self.param_number_id_map.iter().collect::<Vec<_>>();
// 		v.sort_by_key(|(k, _)| *k);
// 		v.into_iter().for_each(|(number, id)| {
// 			let mut line = format!("{}. {}", number, id);

// 			if let Some(name) = param_map.get(id) {
// 				line += " ";
// 				line += &name.name;
// 			}

// 			w.write_all((line + "\n").as_bytes());
// 		});

// 		Ok(())
// 	}

// 	fn plot_param_number_id_map(&self, ax: &mut Axes2D, param_map: &HashMap<String, Parameter>) {
// 		let mut v = self.param_number_id_map.iter().collect::<Vec<_>>();
// 		v.sort_by_key(|(k, _)| *k);
// 		v.into_iter().for_each(|(number, id)| {
// 			let mut label = format!("{}. {}", number, id);

// 			if let Some(name) = param_map.get(id) {
// 				label += " ";
// 				label += &name.name;
// 			}

// 			ax.label(
// 				&label,
// 				Coordinate::Axis(*number as f64),
// 				Coordinate::Graph(-20.0),
// 				&[LabelOption::Rotate(90.0)],
// 			);
// 		});
// 	}

// 	fn plot(self, ax: &mut Axes2D) -> Result<(), String> {
// 		let chars = POINT_SYMBOLS.chars().collect::<Vec<_>>();
// 		let cloneh = self.lang_params_map.clone();
// 		let mut ids = cloneh.keys().collect::<Vec<&String>>();
// 		ids.sort();
// 		let mut lang_params = self.lang_params_map.into_iter().collect::<Vec<_>>();
// 		let id_i = ids
// 			.into_iter()
// 			.enumerate()
// 			.map(|(i, s)| (s, i))
// 			.collect::<HashMap<_, _>>();
// 		lang_params.sort_by_key(|(lang_id, _)| id_i.get(lang_id).unwrap_or(&100));

// 		lang_params
// 			.into_iter()
// 			.enumerate()
// 			.for_each(|(i, (lang_id, param_map))| {
// 				let mut kvs = param_map.into_iter().collect::<Vec<_>>();
// 				kvs.sort_by_key(|(k, _)| *k);
// 				let (xs, ys) = unzip_dict(kvs);

// 				let c = chars[i];
// 				ax.lines_points(
// 					xs,
// 					ys,
// 					&[
// 						PlotOption::PointSymbol(c),
// 						PlotOption::Caption(&lang_id),
// 						PlotOption::PointSize(0.8),
// 					],
// 				);
// 			});

// 		Ok(())
// 	}
// }

impl AnotherGraphSource {
	fn sort_by_distinct_count_max_asc(self) -> Self {
		let paramid_valuelangs = transform_ddict(self.lang_params_map.clone());
		// dbg!(&paramid_valuelangs);

		let mut ps = paramid_valuelangs
			.into_iter()
			.filter_map(|(param_id, h)| {
				let ma = h.keys().max()?;
				Some((param_id, h.len(), *ma))
			})
			.collect::<Vec<_>>();
		ps.sort_by_key(|(_, len, max)| len * 100 + max);
		// dbg!(&ps);

		let param_id_number_map: HashMap<String, usize> = ps
			.into_iter()
			.map(|(k, _, _)| k)
			.enumerate()
			.map(|(i, k)| (k, i))
			.collect();

		Self {
			param_id_number_map,
			lang_params_map: self.lang_params_map,
		}
	}
	
	fn write_param_number_id_map<P: AsRef<Path>>(
		&self,
		path: P,
		param_map: &HashMap<String, Parameter>,
		chapter_map: &HashMap<usize, Chapter>,
	) -> Result<(), String> {
		dbg!(chapter_map);

		let f = OpenOptions::new()
			.truncate(true)
			.create(true)
			.write(true)
			.open(path)
			.map_err(|e| format!("{e} @write_param_number_id_map"))?;
		let mut w = BufWriter::new(f);

		let param_number_id_map = rev_dict(self.param_id_number_map.clone());
		let mut v = param_number_id_map.iter().collect::<Vec<_>>();
		v.sort_by_key(|(k, _)| *k);
		v.into_iter().for_each(|(number, id)| {
			let mut line = format!("{}. ", number);

			if let Some(param) = param_map.get(id) {
				// dbg!(param);
				line = format!("{line}{}", if let Some(chapter) = chapter_map.get(&param.chapter_id)
				&& let Some(url) = chapter.url_in_citation() {
					format!("[{}]({})", &param.name, url)
				} else {
					param.name.clone()
				});
			}

			w.write_all((line + "\n").as_bytes());
		});

		Ok(())
	}

	fn plot_param_number_id_map(&self, ax: &mut Axes2D, param_map: &HashMap<String, Parameter>) {
		let mut v = self.param_id_number_map.iter().collect::<Vec<_>>();
		v.sort_by_key(|(_, k)| *k);
		v.into_iter().for_each(|(id, number)| {
			let mut label = format!("{}. {}", number, id);

			if let Some(param) = param_map.get(id) {
				label += " ";
				label += &param.name;
			}

			ax.label(
				&label,
				Coordinate::Axis(*number as f64),
				Coordinate::Graph(-20.0),
				&[LabelOption::Rotate(90.0)],
			);
		});
	}

	fn plot(self, ax: &mut Axes2D) -> Result<(), String> {
		let chars = POINT_SYMBOLS.chars().collect::<Vec<_>>();
		let cloneh = self.lang_params_map.clone();
		let mut ids = cloneh.keys().collect::<Vec<&String>>();
		ids.sort();
		let mut lang_params = self.lang_params_map.into_iter().collect::<Vec<_>>();
		let id_i = ids
			.into_iter()
			.enumerate()
			.map(|(i, s)| (s, i))
			.collect::<HashMap<_, _>>();
		lang_params.sort_by_key(|(lang_id, _)| id_i.get(lang_id).unwrap_or(&100));

		lang_params
			.into_iter()
			.enumerate()
			.for_each(|(i, (lang_id, param_map))| {
				let mut kvs = param_map
					.into_iter()
					.filter_map(|(id, v)| self.param_id_number_map.get(&id).map(|n| (*n, v)))
					.collect::<Vec<_>>();
				kvs.sort_by_key(|(k, _)| *k);
				let (xs, ys) = unzip_dict(kvs);

				let c = chars[i];
				ax.lines_points(
					xs,
					ys,
					&[
						PlotOption::PointSymbol(c),
						PlotOption::Caption(&lang_id),
						PlotOption::PointSize(0.8),
					],
				);
			});

		Ok(())
	}
}

impl From<Vec<&MyLanguage>> for GraphSource {
	fn from(value: Vec<&MyLanguage>) -> Self {
		let mut param_number_id_map = HashMap::new();
		let mut param_id_number_map = HashMap::new();
		let mut param_number = 0;

		for lang in value.clone() {
			for (param_id, samevalue) in lang.param_values.iter() {
				if let Some(number) = param_id_number_map.get(param_id) {
				} else {
					param_id_number_map.insert(param_id.clone(), param_number);
					param_number_id_map.insert(param_number, param_id.clone());
					param_number += 1;
				}
			}
		}

		let lang_params_map = value
			.into_iter()
			.map(|m| {
				(
					m.language.id.clone(),
					m.param_values
						.iter()
						.filter_map(|(param_id, samevalue)| {
							let i = param_id_number_map.get(param_id)?;
							Some((*i, samevalue.value))
						})
						.collect(),
				)
			})
			.collect();

		Self {
			lang_params_map,
			param_number_id_map,
		}
	}
}

impl From<Vec<&MyLanguage>> for AnotherGraphSource {
	fn from(value: Vec<&MyLanguage>) -> Self {
		let mut param_number_id_map = HashMap::new();
		let mut param_id_number_map = HashMap::new();
		let mut param_number = 0;

		for lang in value.clone() {
			for (param_id, samevalue) in lang.param_values.iter() {
				if let Some(number) = param_id_number_map.get(param_id) {
				} else {
					param_id_number_map.insert(param_id.clone(), param_number);
					param_number_id_map.insert(param_number, param_id.clone());
					param_number += 1;
				}
			}
		}

		let lang_params_map = value
			.into_iter()
			.map(|m| {
				(
					m.language.id.clone(),
					m.param_values
						.iter()
						.map(|(param_id, samevalue)| (param_id.clone(), samevalue.value))
						.collect(),
				)
			})
			.collect();

		Self {
			lang_params_map,
			param_id_number_map,
		}
	}
}
