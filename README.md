# zigbee-analyzer

<p align="center">
  <a href="https://github.com/whisprer/zigbee-analyzer/releases"> 
    <img src="https://img.shields.io/github/v/release/whisprer/zigbee-analyzer?color=4CAF50&label=release" alt="Release"> 
  </a>
  <a href="https://github.com/whisprer/zigbee-analyzer/blob/main/LICENSE"> 
    <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="License"> 
    <img src="http://img.shields.io/badge/license-CC0-green.sg" alt="license">
  </a>
  <img src="https://img.shields.io/badge/rust-1.70%2B-orange.svg" alt="Rust">
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux%20%7C%20macOS-lightgrey.svg" alt="Platform">
</p>

## 🔍 Overview

**zigbee-analyzer** is a production-grade Zigbee network analyzer written in Rust. Real-time packet capture, security analysis, and network monitoring for Zigbee networks.

### Key Features

- 📡 **Real-time Packet Capture** - Zigbee channels 11-26
- 🖥️ **Interactive TUI** - 7 specialized analysis tabs
- 💻 **Powerful CLI** - Scriptable automation tools
- 🔒 **Security Analysis** - Threat detection and encryption monitoring
- 📊 **Network Topology** - Device mapping and relationships
- 🌐 **Channel Analysis** - Interference detection and optimization
- 🚨 **Anomaly Detection** - ML-based threat identification

## 🚀 Quick Start

### Hardware Required

- Zigbee USB dongle (CC2531, CC2652, or ConBee)

### Installation

```bash
git clone https://github.com/whisprer/zigbee-analyzer.git
cd zigbee-analyzer
cargo build --release
```

### Usage

#### Terminal UI
```bash
cargo run --release --bin zigbee-analyzer
```

**Controls:** `1-7` tabs | `↑/↓` scroll | `Space` pause | `Q` quit

#### CLI
```bash
# List devices
zigbee-cli list

# Capture packets
zigbee-cli capture -c 15 -n 1000

# Scan channels
zigbee-cli scan

# Analyze quality
zigbee-cli channels
```

## 📁 Structure

```
crates/
├── zigbee-core/      # Protocol parsing
├── zigbee-hal/       # Hardware abstraction
├── zigbee-drivers/   # Device drivers
├── zigbee-analysis/  # Analysis engines
├── zigbee-ui/        # Terminal UI
└── zigbee-cli/       # CLI tools
```

## 🔧 Supported Devices

| Device | Status |
|--------|--------|
| TI CC2531 | ✅ Supported |
| TI CC2652 | ✅ Supported |
| ConBee/ConBee II | ✅ Supported |

## 🐛 Troubleshooting

**Linux:** Add user to dialout group
```bash
sudo usermod -a -G dialout $USER
```

**Windows:** Check Device Manager for drivers

## 🤝 Contributing

Contributions welcome! See [CONTRIBUTING.md](CONTRIBUTING.md).

## 📄 License

MIT License - see [LICENSE](LICENSE) file.

## 📞 Contact

- **Author:** wofl
- **Organization:** whispr.dev
- **Security:** security@whispr.dev

## ⚠️ Disclaimer

For authorized network administration and security research only.

---

**Built with 🐺 by wofl**
