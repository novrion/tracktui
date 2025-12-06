use std::{collections::HashMap, error::Error, fs::File, path::Path};

use crate::data_series::DataSeries;

pub fn read_csv(path: impl AsRef<Path>) -> Result<Vec<DataSeries>, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut rdr = csv::Reader::from_reader(file);

    let mut series_map: HashMap<String, Vec<(i64, f64)>> = HashMap::new();

    for result in rdr.records() {
        let record = result?;
        let name = record.get(0).ok_or("Missing name")?.to_string();
        let x: i64 = record.get(1).ok_or("Missing x")?.parse()?;
        let y: f64 = record.get(2).ok_or("Missing y")?.parse()?;

        series_map.entry(name).or_default().push((x, y));
    }

    let mut series: Vec<DataSeries> = series_map
        .into_iter()
        .map(|(name, mut data)| {
            data.sort_by_key(|&(x, _)| x);
            DataSeries { name, data }
        })
        .collect();

    series.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(series)
}

pub fn write_csv(data_series: &[DataSeries], path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let mut wtr = csv::Writer::from_writer(file);

    wtr.write_record(&["name", "x", "y"])?;

    for serie in data_series {
        for &(x, y) in &serie.data {
            wtr.write_record(&[serie.name.as_str(), &x.to_string(), &y.to_string()])?;
        }
    }

    wtr.flush()?;
    Ok(())
}
