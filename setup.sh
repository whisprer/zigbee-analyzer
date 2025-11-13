#!/bin/bash
# Run this to create all directories and empty files

# Create directory structure
mkdir -p crates/zigbee-{core,hal,drivers,analysis,ui,cli}/src
mkdir -p docs tests examples

# Create Cargo.toml files (you'll need to fill these)
touch Cargo.toml
touch crates/zigbee-core/Cargo.toml
touch crates/zigbee-hal/Cargo.toml
touch crates/zigbee-drivers/Cargo.toml
touch crates/zigbee-analysis/Cargo.toml
touch crates/zigbee-ui/Cargo.toml
touch crates/zigbee-cli/Cargo.toml

# Create source files
# zigbee-core
touch crates/zigbee-core/src/{lib,packet,ieee802154,network,aps,zcl,crypto,constants}.rs

# zigbee-hal
touch crates/zigbee-hal/src/{lib,traits,capabilities,error,mock}.rs

# zigbee-drivers
touch crates/zigbee-drivers/src/{lib,ti_cc2531,ti_cc2652,conbee,pcap,registry}.rs

# zigbee-analysis
touch crates/zigbee-analysis/src/{lib,topology,statistics,device_db,security,channel,anomaly,export}.rs

# zigbee-ui
mkdir -p crates/zigbee-ui/src/{views,widgets}
touch crates/zigbee-ui/src/{main,app,theme}.rs
touch crates/zigbee-ui/src/views/{mod,capture,topology,packets,analysis,settings}.rs
touch crates/zigbee-ui/src/widgets/{mod,waterfall,graph,hex_viewer}.rs

# zigbee-cli
touch crates/zigbee-cli/src/main.rs

# Docs
touch docs/{architecture,hardware,protocol}.md

# Other
touch README.md LICENSE .gitignore

echo "Project skeleton created!"