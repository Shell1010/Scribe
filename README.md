# Scribe

Scribe is a Rust-based network packet sniffer and real-time combat log parser for AdventureQuest Worlds (AQW). 

By intercepting SmartFoxServer (SFS) traffic on TCP port 5588, Scribe decodes raw byte streams into JSON and maps them to strongly typed Rust structures. It reconstructs the game's combat timeline, tracking absolute vitals, precise damage events, aura applications, and boss mechanics as they happen.

## Features

* Real-Time Vital Tracking: Calculates and streams exact damage numbers, MP costs, and safeguard changes for both players and monsters.
* Action Result Parsing: Extracts direct hit, dodge, and critical strike data directly from the server's action result arrays.
* Aura Monitoring: Tracks the application, refresh, and expiration of buffs and debuffs across all actors in a loaded room.
* Boss Mechanic Detection: Scans animation packets for specific text prompts to identify incoming special attacks or boss shielding phases.
* Dynamic Stat Updates: Captures live backend stat modifiers (e.g., critical power multipliers) as they are assigned by the server.

## Prerequisites
* `libpcap` development headers (Linux/macOS) or Npcap (Windows)

## Getting Started

1. Clone the repository.
2. Ensure you have the necessary pcap dependencies installed for your operating system.
3. Run the application with elevated privileges, required to capture packets:

```bash
cargo build --release
sudo setcap cap_net_raw,cap_net_admin=eip target/release/scribe
./target/release/scribe
```