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
