mod models;

fn write_csv(&data_series: DataSeries, path: String) -> Result<(), Box<dyn Error>> {
        let file = File::create(path)?;
        let mut wtr = csv::Writer::from_writer(file);
        
        wtr.write_record(&["name", "x", "y"])?;
        
        // Flatten: write each data point as a separate row
        for serie in data_series {
            for &(x, y) in &serie.data {
                wtr.write_record(&[
                    serie.name.as_str(),
                    &x.to_string(),
                    &y.to_string(),
                ])?;
            }
        }
        
        wtr.flush()?;
        Ok(())
    }
    
    fn read_csv(&mut self, path: String) -> Result<DataSerie, Box<dyn Error>> {
        let file = File::open(path)?;
        let mut rdr = csv::Reader::from_reader(file);
        
        use std::collections::HashMap;
        let mut series_map: HashMap<String, Vec<(i64, f64)>> = HashMap::new();
        
        for result in rdr.records() {
            let record = result?;
            let name = record.get(0).ok_or("Missing name")?.to_string();
            let x: i64 = record.get(1).ok_or("Missing x")?.parse()?;
            let y: f64 = record.get(2).ok_or("Missing y")?.parse()?;
            
            series_map.entry(name).or_insert_with(Vec::new).push((x, y));
        }
        
        // Convert HashMap to Vec<DataSeries>
        for (name, mut data) in series_map {
            data.sort_by(|a, b| a.0.cmp(&b.0));
            self.data_series.push(DataSeries { name, data });
        }
        
        Ok(())
    }
