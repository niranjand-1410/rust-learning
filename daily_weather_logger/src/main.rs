use serde::Deserialize;
use std::fs::OpenOptions;
use std::path::Path;
use clap::Parser;



#[derive(Deserialize, Debug)]
pub struct WeatherData {
    pub latitude: f64,
    pub longitude: f64,
    pub generationtime_ms: f64,
    pub utc_offset_seconds: i32,
    pub timezone: String,
    pub timezone_abbreviation: String,
    pub elevation: f64,
    pub current_units: CurrentUnits,
    pub current: CurrentData,
}

#[derive(Deserialize, Debug)]
pub struct CurrentUnits {
    pub time: String,
    pub interval: String,
    pub temperature_2m: String,
}

#[derive(Deserialize, Debug)]
pub struct CurrentData {
    pub time: u64,
    pub interval: u32,
    pub temperature_2m: f64,
}


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    latitude: f32,
    #[arg(long)]
    longitude: f32,
}


fn append_weather_to_csv(path: &str, data: &WeatherData) -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new(path);
    let file_exists = path.exists();
    let file_is_empty = if file_exists {
        std::fs::metadata(path)?.len() == 0
    } else {
        true
    };

    // Open file in append mode, create it if it does not exist
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)?;

    // Build the CSV writer config
    let mut wtr = csv::WriterBuilder::new()
        .has_headers(false) // Handle headers manually to prevent repeats during appending
        .from_writer(file);

    // Write headers only if the file is brand new or empty
    if file_is_empty {
        wtr.write_record(&[
            "latitude",
            "longitude",
            "generationtime_ms",
            "utc_offset_seconds",
            "timezone",
            "timezone_abbreviation",
            "elevation",
            "current_time",
            "current_interval",
            "current_temperature_2m",
        ])?;
    }

    // Append the flattened structural fields
    wtr.write_record(&[
        data.latitude.to_string(),
        data.longitude.to_string(),
        data.generationtime_ms.to_string(),
        data.utc_offset_seconds.to_string(),
        data.timezone.clone(),
        data.timezone_abbreviation.clone(),
        data.elevation.to_string(),
        data.current.time.to_string(),
        data.current.interval.to_string(),
        data.current.temperature_2m.to_string(),
    ])?;

    wtr.flush()?;
    Ok(())
}

#[tokio::main]
async fn main()  -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("Fetching data for latitude: {} longitude: {}",args.latitude, args.longitude);

    let api_url = format!("https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m&timezone=GMT&forecast_days=1&timeformat=unixtime",args.latitude, args.longitude);
    let resp = reqwest::get(api_url).await?.json::<WeatherData>().await?;
    println!("{:?}",resp);
    append_weather_to_csv("data.csv",&resp)
}
