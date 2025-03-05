use anyhow::Error;
use metrics::{counter, describe_counter};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::process::Command;
use std::time::Duration;
use std::{env, thread};

enum CameraType {
    Rtsp,
    Http,
}

impl TryFrom<String> for CameraType {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "RTSP" => Ok(CameraType::Rtsp),
            "HTTP" => Ok(CameraType::Http),
            _ => Err(Error::msg("invalid camera type")),
        }
    }
}

enum Camera {
    Rtsp {
        rtsp_url: String,
    },
    Http {
        http_url: String,
        client: reqwest::blocking::Client,
    },
}

impl Camera {
    fn fetch_snapshot(&self, output_folder: &str) -> Result<(), Error> {
        match self {
            Camera::Rtsp { rtsp_url } => {
                // Write bytes to file in output file based on current timestamp.
                let timestamp = chrono::Utc::now().timestamp();
                let output_file = format!("{}/{}.jpg", output_folder, timestamp);
                let output = Command::new("ffmpeg")
                    .arg("-y")
                    .arg("-rtsp_transport")
                    .arg("tcp")
                    .arg("-i")
                    .arg(rtsp_url)
                    .arg("-vframes")
                    .arg("1")
                    .arg(output_file)
                    .output()?;

                if !output.status.success() {
                    return Err(Error::msg("Failed to fetch snapshot from RTSP camera"));
                }

                Ok(())
            }
            Camera::Http { http_url, client } => {
                // Write bytes to file in output file based on current timestamp.
                let timestamp = chrono::Utc::now().timestamp();
                let output_file = format!("{}/{}.jpg", output_folder, timestamp);

                let bytes = client.get(http_url).send()?.bytes()?;

                // Check if the received frame is the "Preview Not Available" frame.
                let md5 = format!("{:x}", md5::compute(bytes.clone()));
                if md5 == "64a9507f752d598345378763b25bdcaf" {
                    return Err(Error::msg("received Preview Not Available frame"));
                }

                std::fs::write(output_file, bytes)?;

                Ok(())
            }
        }
    }
}

fn main() {
    let feed_url = env::var("FEED_URL").expect("FEED_URL environment variable must be set");
    let feed_name = env::var("FEED_NAME").unwrap_or(feed_url.clone());
    let output_folder =
        env::var("OUTPUT_FOLDER").expect("OUTPUT_FOLDER environment variable must be set");
    let camera_type: CameraType = env::var("CAMERA_TYPE")
        .expect("CAMERA_TYPE environment variable must be set")
        .try_into()
        .expect("Invalid camera type");
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
    let counter = counter!("timelapse_snapshot_count", "feed_name" => feed_name.clone());

    let camera = match camera_type {
        CameraType::Rtsp => Camera::Rtsp { rtsp_url: feed_url },
        CameraType::Http => Camera::Http {
            http_url: feed_url,
            client: reqwest::blocking::Client::new(),
        },
    };

    loop {
        println!("Fetching snapshot...");
        match camera.fetch_snapshot(&output_folder) {
            Ok(_) => {
                counter.increment(1);
                println!("Snapshot fetched successfully.");
            }
            Err(e) => eprintln!("Error fetching snapshot: {}", e),
        }

        println!("Sleeping...");
        thread::sleep(Duration::from_secs(sleep_secs));
    }
}
