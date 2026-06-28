use serde::Deserialize;
use scribe_sniffer::ScribeSniffer;
use scribe_core::IdentityMapper;
use scribe_parser::ScribeParser;
use std::sync::mpsc;

#[derive(Deserialize)]
struct Settings {
    port: u16,
    device_name: Option<String>,
}

fn main() {
    let settings: Settings = config::Config::builder()
        .add_source(config::File::with_name("config"))
        .build().unwrap().try_deserialize().unwrap();
        
    let port = settings.port;
    let device = settings.device_name.filter(|s| !s.is_empty());

    run_tui_mode(port, device);
}

fn run_tui_mode(port: u16, device: Option<String>) {
    let (tx, rx) = mpsc::channel();
    
    std::thread::spawn(move || {
        if let Ok(mut sniffer) = ScribeSniffer::new(device.as_deref(), port) {
            loop {
                for j in sniffer.next_json_objects() {
                    if tx.send(j).is_err() { break; }
                }
            }
        }
    });

    let mapper = IdentityMapper::new();

    let mut parser = ScribeParser::new(mapper);
    let app = tui::app::App::new(Some("output.txt"));
    let mut terminal = tui::setup_terminal().unwrap();
    
    let _ = tui::run_app(&mut terminal, app, rx, &mut parser);
    tui::restore_terminal(terminal).unwrap();
}

/*
#[tokio::main]
async fn run_cli_mode(port: u16, device: Option<String>) {
    use scribe_output::ScribeOutput;
    use tokio::sync::{mpsc as tmpsc, Mutex};
    use std::sync::Arc;

    let (tx, mut rx) = tmpsc::channel::<String>(1000);
    let mapper = IdentityMapper::new();
    mapper.load_from_disk();
    let parser = Arc::new(ScribeParser::new(mapper));
    let output = Arc::new(Mutex::new(ScribeOutput::new("output.txt")));

    tokio::task::spawn_blocking(move || {
        if let Ok(mut sniffer) = ScribeSniffer::new(device.as_deref(), port) {
            loop {
                for j in sniffer.next_json_objects() {
                    if tx.blocking_send(j).is_err() { break; }
                }
            }
        }
    });

    while let Some(raw_json) = rx.recv().await {
        let p = Arc::clone(&parser);
        let o = Arc::clone(&output);
        tokio::spawn(async move {
            let events = p.parse_packet(&raw_json);
            if !events.is_empty() {
                let mut out = o.lock().await;
                for e in events { out.handle_event(e); }
            }
        });
    }
}
*/