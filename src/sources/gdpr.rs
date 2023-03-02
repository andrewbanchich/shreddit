use csv::Reader;
use serde::de::DeserializeOwned;
use std::path::Path;

pub trait Gdpr {
    const FILENAME: &'static str;
}

pub fn list<T>(export_dir: &Path) -> impl Iterator<Item = T>
where
    T: Gdpr + DeserializeOwned,
{
    let mut p = export_dir.to_path_buf();
    p.push(T::FILENAME);

    let things = Reader::from_path(p).unwrap();
    things.into_deserialize().map(|f| f.unwrap())
}
