# Scribe

Scribe is a Rust-based network packet sniffer for AdventureQuest Worlds (AQW). Utility tool that is not a bot.

## Features
- Combat Metrics, track KPM, GPM, EPM, Drops, Drop Rates (assumed).
- View Class Metadata, including Skills, Passives, and Potions/Scrolls.
- Aura tracking.
- Stat Updates.
- Damage tracking.
- Logs outputs to an `output.log` file to view combat breakdown.

## Prerequisites
* `libpcap` development headers (Linux/macOS) or Npcap (Windows).

## Getting Started
1. Download the latest release from the [Releases](https://github.com/Shell1010/Scribe/releases) page.
2. Create a `config.toml` in the same folder, with the following configuration

```toml
port = [5594, 5588] # 5594/5588
device_name = ""
```

3. Copy the [mechanics.json](mechanics.json) file to the same folder.
4. Run the executable, yell at me if it doesn't work.

## Mechanics/Ultras
This script provides an indicator for Ultras, I've made a configuration system that allows you to setup indicators for a variety of Ultras. It parses the information in `mechanics.json` to determine what to track and how. Currently somewhat limited functionality, but I intend to improve it. Documentation for this feature is provided [here](docs/mechanics.md). Kinda requires you to know how the JSON format works, it's not very complex.

## Building from Source
1. Clone the repository.
2. Ensure you have the necessary pcap dependencies installed for your operating system.

3. Create a `config.toml` in the same folder, with the following configuration

```toml
port = [5594, 5588] # 5594/5588
device_name = ""
```

4. Build the thingamajig

```bash
cargo build --release
sudo setcap cap_net_raw,cap_net_admin=eip target/release/scribe
./target/release/scribe
```



