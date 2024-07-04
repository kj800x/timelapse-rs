use anyhow::Error;
use metrics::{counter, describe_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use reqwest::blocking::Client;
use std::time::Duration;
use std::{env, thread};

fn fetch_snapshot(
    client: &Client,
    feed_url: &str,
    output_folder: &str,
) -> Result<(), anyhow::Error> {
    let bytes = client.get(feed_url).send()?.bytes()?;

    // Check if the received frame is the "Preview Not Available" frame.
    let md5 = format!("{:x}", md5::compute(bytes.clone()));
    if md5 == "64a9507f752d598345378763b25bdcaf" {
        return Err(Error::msg("received Preview Not Available frame"));
    }

    // Write bytes to file in output file based on current timestamp.
    let timestamp = chrono::Utc::now().timestamp();
    let output_file = format!("{}/{}.jpg", output_folder, timestamp);
    std::fs::write(output_file, bytes)?;

    Ok(())
}

fn main() {
    let feed_url = env::var("FEED_URL").expect("FEED_URL environment variable must be set");
    let output_folder =
        env::var("OUTPUT_FOLDER").expect("OUTPUT_FOLDER environment variable must be set");
    let sleep_secs: u64 = env::var("SLEEP_SECS")
        .unwrap_or("900".to_string())
        .parse()
        .expect("SLEEP_SECS must be set to a valid number");
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(([0, 0, 0, 0], 9090))
        .install()
        .expect("Failed to install Prometheus recorder");
    describe_counter!(
        "timelapse_snapshot_count",
        "The number of snapshots taken by the timelapse service."
    );
    let counter = counter!("timelapse_snapshot_count", "feed_url" => feed_url.clone());
    let client = reqwest::blocking::Client::new();

    loop {
        println!("Fetching snapshot...");
        match fetch_snapshot(&client, &feed_url, &output_folder) {
            Ok(_) => {
                counter.increment(1);
                println!("Snapshot fetched successfully.");
            }
            Err(e) => eprintln!("Error fetching snapshot: {}", e),
        }

        // Sleep 1 hour.
        println!("Sleeping...");
        thread::sleep(Duration::from_secs(sleep_secs));
    }
}
