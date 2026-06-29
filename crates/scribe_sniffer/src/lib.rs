use pcap::{Capture, Device};
use etherparse::{PacketHeaders, PayloadSlice, TransportHeader};
use std::collections::HashMap;


mod sfs;
use sfs::SfsSession;



pub struct ScribeSniffer {
    capture: Capture<pcap::Active>,
    sessions: HashMap<String, SfsSession>,
    
}

impl ScribeSniffer {
    pub fn new(device_name: Option<&str>, port: Vec<u16>) -> Result<Self, pcap::Error> {
        let device = match device_name {
            Some(name) => Device::list()?.into_iter().find(|d| d.name == name),
            None => Device::lookup()?,
        }.expect("Could not find sniff, device lookup fail.");

        let mut capture = Capture::from_device(device)?
            .promisc(true)
            .snaplen(65535)
            .immediate_mode(true)
            .open()?;
        
        let filter_str = port
            .iter()
            .map(|p| format!("tcp port {}", p))
            .collect::<Vec<String>>()
            .join(" or ");
        capture.filter(&filter_str, true)?;

        Ok(Self {
            capture,
            sessions: HashMap::new(),
        })
        
    }

    pub fn next_json_objects(&mut self) -> Vec<String> {
        let mut objects = Vec::new();
        
        if let Ok(packet) = self.capture.next_packet() && let Ok(headers) = PacketHeaders::from_ethernet_slice(packet.data) && let (Some(net_header), Some(transport)) = (headers.net, headers.transport) && let TransportHeader::Tcp(tcp) = transport && let etherparse::NetHeaders::Ipv4(ipv4_header, _extensions) = net_header {
            let session_key = format!(
                "{:?}:{}-{:?}:{}", 
                ipv4_header.source, 
                tcp.source_port, 
                ipv4_header.destination, 
                tcp.destination_port
            );
    
            let session = self.sessions.entry(session_key).or_insert_with(SfsSession::new);
    
    
    
            match headers.payload {
                PayloadSlice::Empty => {}
                PayloadSlice::Tcp(payload) => {
                    let extracted = session.process(payload);
                    objects.extend(extracted);
                },
                _ => {}
            }
        }
        
        objects
    }
}