use scribe_sniffer::ScribeSniffer;
use scribe_core::IdentityMapper;
use scribe_parser::ScribeParser;
use scribe_output::ScribeOutput;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    println!("Initializing Scribe");

    let identity_mapper = IdentityMapper::new();
    let (tx, mut rx) = mpsc::channel::<String>(10_000);
    let target_device = None;
    let target_port = 5588;

    
    let parser = Arc::new(ScribeParser::new(identity_mapper));
    let output = Arc::new(Mutex::new(ScribeOutput::new("output.txt")));

    println!("Listening for game traffic. Enter a room to cache profiles...");

    let _sniffer_task = tokio::task::spawn_blocking(move || {
        let mut sniffer = ScribeSniffer::new(target_device, target_port).expect("Failed to bind sniffer");
        
        loop {
            let json_objects = sniffer.next_json_objects();
            
            for json_str in json_objects {
                if tx.blocking_send(json_str).is_err() {
                    println!("Receiver closed, shutting down sniffer.");
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