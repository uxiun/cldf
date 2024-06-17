use std::{collections::HashMap, hash::Hash};

pub fn flatten<T, E>(result: Result<Result<T, E>, E>) -> Result<T, E> {
	match result {
		Ok(r) => r,
		Err(e) => Err(e),
	}
}

pub fn all_ok<T, E, I>(results: I) -> Result<Vec<T>, E>
where
	I: Iterator<Item = Result<T, E>>,
{
	let mut v = vec![];
	for result in results {
		match result {
			Ok(t) => v.push(t),
			Err(e) => return Err(e),
		}
	}

	Ok(v)
}

pub fn unzip_dict<K, V, I>(dict: I) -> (Vec<K>, Vec<V>)
where
	I: IntoIterator<Item = (K, V)>,
{
	let (mut ks, mut vs) = (vec![], vec![]);

	for (k, v) in dict {
		ks.push(k);
		vs.push(v);
	}

	(ks, vs)
}

pub fn rev_dict<K, V, I>(h: I) -> HashMap<V, K>
where
	K: Hash + Eq,
	V: Hash + Eq,
	I: IntoIterator<Item = (K, V)>,
{
	h.into_iter().map(|(k, v)| (v, k)).collect()
}

pub fn transform_ddict<J, K, V, I>(h: I) -> HashMap<K, HashMap<V, Vec<J>>>
where
	I: IntoIterator<Item = (J, HashMap<K, V>)>,
	K: Eq + Hash,
	V: Eq + Hash + Clone,
	J: Eq + Hash + Clone,
{
	let mut t: HashMap<K, HashMap<V, Vec<J>>> = HashMap::new();

	for (j, kv) in h {
		for (k, v) in kv {
			t.entry(k)
				.and_modify(|vj| {
					vj.entry(v.clone())
						.and_modify(|js| js.push(j.clone()))
						.or_insert(vec![j.clone()]);
				})
				.or_insert(HashMap::from_iter([(v, vec![j.clone()])]));
		}
	}

	t
}

pub fn map_keys_dict<K, V, U>(h: HashMap<K, V>, mapper: &HashMap<K, U>) -> HashMap<U, V>
where
	K: Hash + Eq,
	U: Hash + Eq + Clone,
{
	h.into_iter()
		.filter_map(|(k, v)| {
			let u = mapper.get(&k)?;
			Some((u.clone(), v))
		})
		.collect()
}

pub fn map_map_dict<K, V, U>(h: HashMap<K, V>, g: &HashMap<V, U>) -> HashMap<K, U>
where
	K: Hash + Eq,
	V: Hash + Eq,
	U: Clone,
{
	h.into_iter()
		.filter_map(|(k, v)| {
			let u = g.get(&v)?;
			Some((k, u.clone()))
		})
		.collect()
}

#[test]
fn transform() {
	let h: HashMap<&str, HashMap<&str, usize>> = [
		("apple", [("color", 1), ("like", 5)].into_iter().collect()),
		("banana", [("color", 3), ("like", 3)].into_iter().collect()),
		("orange", [("color", 2), ("like", 3)].into_iter().collect()),
	]
	.into_iter()
	.collect();

	let g = transform_ddict(h);
	dbg!(g);
}

#[test]
fn mapping() {
	let h: HashMap<usize, usize> = HashMap::from_iter([(1, 2), (2, 3), (3, 4)]);

	let g: HashMap<usize, usize> = HashMap::from_iter([(1, 0), (2, 1), (3, 2)]);

	let hg = map_keys_dict(h.clone(), &g);
	dbg!(hg);

	let mm = dbg!(map_map_dict(h, &g));
}
