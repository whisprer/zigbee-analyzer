# Zigbee CLI - Command Line Interface

A powerful command-line tool for Zigbee network analysis and packet capture.

## Installation

Copy these files to your project:

```bash
cp zigbee_cli_main.rs crates/zigbee-cli/src/main.rs
cp zigbee_cli_Cargo.toml crates/zigbee-cli/Cargo.toml
```

Then rebuild:
```bash
cargo build --release --bin zigbee-cli
```

## Commands

### 1. List Available Devices

Find all connected Zigbee USB dongles:

```bash
zigbee-cli list
```

**Output:**
```
🔍 Scanning for Zigbee devices...

Found 1 device(s):

  [0] TI CC2531 (COM3)
      Type: CC2531
```

### 2. Capture Packets

Capture packets from a Zigbee network:

```bash
# Capture 100 packets on channel 15
zigbee-cli capture -c 15 -n 100

# Capture forever on channel 11 (default)
zigbee-cli capture

# Show stats every 10 seconds instead of 5
zigbee-cli capture -c 15 -s 10
```

**Options:**
- `-c, --channel <CHANNEL>` - Zigbee channel (11-26, default: 11)
- `-n, --count <COUNT>` - Number of packets (0 = infinite, default: 0)
- `-s, --stats-interval <SECS>` - Stats display interval (default: 5)

**Note:** File output (`-o` flag) is temporarily disabled until serialization support is added.

**Example output:**
```
📡 Capturing on channel 15 (2.4 GHz)
📊 Capturing packets (Ctrl+C to stop)...

[1] RSSI: -45 dBm | LQI: 255 | Size:  27 bytes
[2] RSSI: -52 dBm | LQI: 240 | Size:  15 bytes
[3] RSSI: -48 dBm | LQI: 250 | Size:  32 bytes

============================================================
📊 STATISTICS
============================================================

📦 Traffic:
  Total Packets:     234
  Total Bytes:       6.45 KB
  Packets/sec:       46.8
  Avg Packet Size:   28.2 bytes

🌐 Network:
  Devices:           12
  Networks:          2
  Links:             18

🔒 Security:
  Threats:           0
  Encrypted Devices: 10
  Unencrypted:       2
============================================================
```

### 3. Scan All Channels

Quick scan of all Zigbee channels to see activity:

```bash
# Scan for 2 seconds per channel (default)
zigbee-cli scan

# Scan for 5 seconds per channel
zigbee-cli scan -d 5
```

**Output:**
```
🔍 Scanning all Zigbee channels...

Channel | Packets | Avg RSSI | Devices
--------|---------|----------|--------
  11    |     123 |  -48 dBm |       5
  12    |       0 |    0 dBm |       0
  13    |       4 |  -62 dBm |       2
  14    |       0 |    0 dBm |       0
  15    |     567 |  -44 dBm |      12
  16    |      12 |  -58 dBm |       3
  ...
  26    |       0 |    0 dBm |       0

✅ Scan complete!
```

### 4. Analyze Channel Quality

Deep analysis of channel quality and interference:

```bash
# Analyze for 30 seconds (default)
zigbee-cli channels

# Analyze for 60 seconds
zigbee-cli channels -d 60
```

**Output:**
```
📊 Analyzing channel quality for 30 seconds...

🏆 Best channel: 15

Channel | Utilization | Avg RSSI | Packets | Quality
--------|-------------|----------|---------|--------
  11    |  45.2%      |  -48 dBm |    2345 | Fair
  12    |   0.5%      |  -72 dBm |      23 | Good
  13    |  12.3%      |  -55 dBm |     678 | Good
  14    |   1.2%      |  -68 dBm |      45 | Good
  15    |  78.9%      |  -42 dBm |    5678 | Poor
  16    |  23.4%      |  -51 dBm |    1234 | Good
  ...
  26    |   0.1%      |  -80 dBm |       5 | Good

✅ Analysis complete!
```

## Use Cases

### 1. Quick Network Check
```bash
# See if there's any Zigbee activity
zigbee-cli scan -d 1
```

### 2. Find Best Channel for Your Network
```bash
# Analyze all channels and pick the least busy one
zigbee-cli channels -d 60
```

### 3. Monitor Network Traffic
```bash
# Watch live traffic on your network's channel
zigbee-cli capture -c 20 -s 2
```

## Requirements

- Zigbee USB dongle (CC2531, CC2652, ConBee, etc.)
- Windows/Linux/macOS
- Rust 1.70+

## Supported Devices

- ✅ Texas Instruments CC2531
- ✅ Texas Instruments CC2652
- ✅ ConBee/ConBee II
- ✅ Other Zigbee dongles with standard drivers

## Tips

1. **Finding Active Networks:** Run `scan` first to see which channels are busy
2. **Best Performance:** Use `--release` builds for production use
3. **Interference Detection:** Use `channels` to find WiFi/microwave interference
4. **Real-time Monitoring:** Lower `-s` interval for faster stats updates

## Troubleshooting

**"No Zigbee devices found!"**
- Make sure your USB dongle is plugged in
- Check device manager (Windows) or `lsusb` (Linux)
- Try a different USB port
- Install device drivers if needed

**"Invalid channel"**
- Zigbee only uses channels 11-26
- Check your network's configured channel

**Permission denied (Linux)**
- Run with sudo: `sudo zigbee-cli list`
- Or add your user to dialout group: `sudo usermod -a -G dialout $USER`

## Examples

```bash
# Full workflow: scan, analyze, capture
zigbee-cli list                    # Find devices
zigbee-cli scan                    # Find active channels
zigbee-cli channels -d 60          # Analyze channel quality
zigbee-cli capture -c 15 -n 10000  # Capture on best channel
```

Enjoy analyzing Zigbee networks! 🐺
