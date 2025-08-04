use radix_fmt::radix;
use rand::random;
use std::collections::HashMap;
use std::env::consts::{ARCH, OS};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tracing_batteries::prelude::*;
use tracing_batteries::{Battery, BatteryBuilder, Metadata};

pub struct MedamaAnalytics {
    server: String,
}

impl MedamaAnalytics {
    pub fn new<S: ToString>(server: S) -> Self {
        Self {
            server: server.to_string(),
        }
    }

    fn generate_beacon_id() -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let uniqueness: u64 = random();

        format!("{}{}", radix(timestamp, 36), radix(uniqueness, 36))
    }

    fn generate_user_agent(service: &str, version: &str) -> String {
        let os_info = match (OS, ARCH) {
            ("macos", "x86_64") => "Macintosh; Intel Mac OS X",
            ("macos", "aarch64") => "Macintosh; Apple Mac OS X",
            ("windows", _) => "Windows NT",
            ("linux", _) => "X11; Linux",
            _ => "Unknown OS",
        };

        format!("Mozilla/5.0 ({os_info}) Gecko/20100101 {service}/{version}")
    }
}

impl BatteryBuilder for MedamaAnalytics {
    fn setup(self, metadata: &Metadata, enabled: Arc<AtomicBool>) -> Box<dyn Battery> {
        let battery = MedamaAnalyticsBattery {
            server: self.server,
            service: metadata.service.to_string(),
            user_agent: Self::generate_user_agent(&metadata.service, &metadata.version),
            beacon_id: Self::generate_beacon_id(),
            start_time: chrono::Utc::now(),
            is_enabled: enabled,
            outstanding_requests: Arc::new(AtomicUsize::new(0)),
            client: Arc::new(reqwest::Client::new()),
        };

        // Spawn the load beacon as a background task
        battery.send_load_beacon(metadata);

        Box::new(battery)
    }
}

#[derive(Clone)]
struct MedamaAnalyticsBattery {
    server: String,
    service: String,
    user_agent: String,
    beacon_id: String,
    start_time: chrono::DateTime<chrono::Utc>,
    is_enabled: Arc<AtomicBool>,
    outstanding_requests: Arc<AtomicUsize>,
    client: Arc<reqwest::Client>,
}

impl Battery for MedamaAnalyticsBattery {
    fn record_error(&self, error: &dyn std::error::Error) {
        let mut data = HashMap::new();
        data.insert("error.message".to_string(), error.to_string());

        self.send_custom_event(data);
    }

    fn shutdown(&mut self) {
        // Spawn the unload beacon as a background task
        self.send_unload_beacon();

        // Wait for all outstanding requests to complete
        self.wait_for_outstanding_requests(Duration::from_secs(5));
    }
}

impl MedamaAnalyticsBattery {
    fn wait_for_outstanding_requests(&self, timeout: Duration) {
        // Wait for up to 5 seconds for outstanding requests to complete
        let start_time = std::time::Instant::now();

        while self.outstanding_requests.load(Ordering::Relaxed) > 0 {
            if start_time.elapsed() >= timeout {
                tracing::warn!("Timeout waiting for outstanding requests to complete");
                break;
            }

            std::thread::sleep(Duration::from_millis(50));
        }
    }

    fn send_load_beacon(&self, metadata: &Metadata) {
        let mut data: HashMap<String, String> = metadata
            .context
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        data.insert("service.name".to_string(), metadata.service.to_string());
        data.insert("service.version".to_string(), metadata.version.to_string());

        let payload = MedamaLoadBeacon {
            b: self.beacon_id.clone(),
            e: "load".to_string(),
            u: format!(
                "https://{}.app/{}?utm_source={OS}&utm_medium={ARCH}",
                metadata.service.to_lowercase(),
                metadata.version,
            ),
            r: "".into(), // Referrer can be set if available
            p: true,
            q: true,
            t: iana_time_zone::get_timezone().unwrap_or_default(),
            d: data,
        };

        self.send_request("api/event/hit", payload);
    }

    fn send_unload_beacon(&self) {
        let duration = chrono::Utc::now()
            .signed_duration_since(self.start_time)
            .num_milliseconds() as u64;

        let payload = MedamaUnloadBeacon {
            b: self.beacon_id.clone(),
            e: "unload".to_string(),
            m: duration,
        };

        self.send_request("api/event/hit", payload);
    }

    fn send_custom_event(&self, data: HashMap<String, String>) {
        let payload = MedamaCustomEvent {
            b: self.beacon_id.clone(),
            e: "custom".to_string(),
            g: format!("{}.app", self.service.to_lowercase()),
            d: data,
        };

        self.send_request("api/event/hit", payload);
    }

    fn send_request<P: serde::Serialize + Send + 'static>(&self, path: &str, payload: P) {
        if !self.is_enabled.load(Ordering::Relaxed) {
            return;
        }

        // Increment the outstanding requests counter
        self.outstanding_requests.fetch_add(1, Ordering::Relaxed);

        let url = format!("{}/{}", self.server, path);

        let client = self.client.clone();
        let outstanding_requests = self.outstanding_requests.clone();
        let user_agent = self.user_agent.clone();
        tokio::spawn(async move {
            let result = client
                .post(&url)
                .json(&payload)
                .header("User-Agent", user_agent)
                .header(
                    "Accept-Language",
                    sys_locale::get_locale().unwrap_or_else(|| "en".to_string()),
                )
                .header("Content-Type", "text/plain")
                .send()
                .await;

            // Decrement the outstanding requests counter when done
            outstanding_requests.fetch_sub(1, Ordering::Relaxed);

            match result {
                Ok(response) => {
                    if !response.status().is_success() {
                        tracing::warn!("Medama request failed: {}", response.status());
                    }
                }
                Err(e) => {
                    // Log the error but do not crash the application
                    tracing::warn!("Error sending Medama event: {}", e);
                }
            }
        });
    }
}

#[derive(serde::Serialize)]
struct MedamaLoadBeacon {
    // The beacon ID for this event
    pub b: String,
    // The event type being sent
    pub e: String,
    // The URL of the page being tracked
    pub u: String,
    // The referrer URL
    pub r: String,
    // Whether the user is unique or not
    pub p: bool,
    // Whether this is the user's first visit
    pub q: bool,
    // The user's timezone (used for location detection)
    pub t: String,
    // The data payload for the event
    pub d: HashMap<String, String>,
}

#[derive(serde::Serialize)]
struct MedamaUnloadBeacon {
    // The beacon ID for this event
    pub b: String,
    // The event type being sent
    pub e: String,
    // The time spent on the page, in milliseconds
    pub m: u64,
}

#[derive(serde::Serialize)]
struct MedamaCustomEvent {
    // The beacon ID for this event
    pub b: String,
    // The event type being sent
    pub e: String,
    // The group name for the event (the hostname of the app)
    pub g: String,
    // The data payload for the event
    pub d: HashMap<String, String>,
}
