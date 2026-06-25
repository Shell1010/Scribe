use scribe_sniffer::ScribeSniffer;
use scribe_output::ScribeOutput;
use std::sync::Arc;
use scribe_core::IdentityMapper;
use scribe_parser::ScribeParser;
use tokio::sync::{mpsc, Mutex};
use serde::Deserialize;

#[derive(Deserialize)]
struct Settings {
    port: u16,
    device_name: Option<String>,
}

#[tokio::main]
async fn main() {
    // 1. Load Configuration
    let settings: Settings = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .expect("Failed to load config.toml")
        .try_deserialize()
        .expect("Config format is invalid");

    let target_port = settings.port;
    let target_device = settings.device_name.filter(|s| !s.is_empty());
    let identity_mapper = IdentityMapper::new();
    let (tx, mut rx) = mpsc::channel::<String>(1000);


    
    let parser = Arc::new(ScribeParser::new(identity_mapper));
    let output = Arc::new(Mutex::new(ScribeOutput::new("output.txt")));

    println!("Listening for game traffic. Enter a room to cache profiles...");

    let _sniffer_task = tokio::task::spawn_blocking(move || {
        let mut sniffer = ScribeSniffer::new(target_device.as_deref(), target_port)
            .expect("Failed to bind sniffer");
        
        loop {
            for json_str in sniffer.next_json_objects() {
                // If the UI thread closes, we stop the sniffer
                if tx.blocking_send(json_str).is_err() {
                    break; 
                }
            }
        }
    });

    while let Some(raw_json) = rx.recv().await {
        
        let parser_clone = Arc::clone(&parser);
        let output_clone = Arc::clone(&output);

        tokio::spawn(async move {
            let events = parser_clone.parse_packet(&raw_json);
            if !events.is_empty() {
                let mut out = output_clone.lock().await;
                
                for event in events {
                    out.handle_event(event);
                }
            }
        });
    }
}