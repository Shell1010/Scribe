use pcap::{Capture, Device};
use etherparse::{PacketHeaders, PayloadSlice, TransportHeader};
use std::collections::HashMap;


mod sfs;
use sfs::SfsSession;

// Use a tuple key (src_ip, src_port, dst_ip, dst_port) instead of format!()
// to avoid heap allocation per packet. u32+u16+u32+u16 = 12 bytes on stack.
type SessionKey = (u32, u16, u32, u16);

pub struct ScribeSniffer {
    capture: Capture<pcap::Active>,
    sessions: HashMap<SessionKey, SfsSession>,
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

        if let Ok(packet) = self.capture.next_packet()
            && let Ok(headers) = PacketHeaders::from_ethernet_slice(packet.data)
            && let (Some(net_header), Some(transport)) = (headers.net, headers.transport)
            && let TransportHeader::Tcp(tcp) = transport
            && let etherparse::NetHeaders::Ipv4(ipv4, _) = net_header
        {
            // Build a stack-allocated session key — zero heap allocation.
            let key: SessionKey = (
                u32::from_ne_bytes(ipv4.source),
                tcp.source_port,
                u32::from_ne_bytes(ipv4.destination),
                tcp.destination_port,
            );

            let session = self.sessions.entry(key).or_insert_with(SfsSession::new);

            if let PayloadSlice::Tcp(payload) = headers.payload {
                objects.extend(session.process(payload));
            }
        }

        objects
    }
}
