use metrics::{counter, describe_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::time::Duration;
use std::{env, thread};

fn main() {
    let feed_url = env::var("FEED_URL").expect("FEED_URL environment variable must be set");
    let output_folder =
        env::var("OUTPUT_FOLDER").expect("OUTPUT_FOLDER environment variable must be set");
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
        counter.increment(1);

        let bytes = client
            .get(feed_url.clone())
            .send()
            .unwrap()
            .bytes()
            .unwrap();

        // Write bytes to file in output file based on current timestamp.
        let timestamp = chrono::Utc::now().timestamp();
        let output_file = format!("{}/{}.jpg", output_folder, timestamp);
        std::fs::write(output_file, bytes).unwrap();

        // Sleep 1 hour.
        println!("Sleeping...");
        thread::sleep(Duration::from_secs(60 /* * 60*/));
    }
}
