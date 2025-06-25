# PCB Parts Client

A comprehensive parts inventory management system. This application helps you organize, track, and manage electronic components, BOMs (Bill of Materials), and stock levels with both a GUI and CLI interface.

![An image of the parts manager](/assets/screenshot.png)

The system is built to work with a gridfinity-based arrangement of bins to store the parts.


## Features

### GUI Application
- **Modern Interface**: Built with [Iced](https://github.com/iced-rs/iced)
- **Part Search & Management**: Search and filter parts by name and description
- **Visual Grid Layout**: Interactive grid widget for organizing parts in physical bins
- **Keyboard Shortcuts**: Configurable keybindings for efficient workflow

### CLI Application  
- **Complete Command Interface**: Full feature parity with GUI for automation
- **Batch Operations**: Perfect for scripting and bulk operations
- **CSV Import**: Import BOMs directly from command line

### Core Functionality
- **Part Database**: Comprehensive part information with descriptions
- **BOM Management**: Create, import, and track bills of materials
- **Stock Control**: Track quantities and physical bin locations (row, column, z-axis)
- **User Management**: Multi-user system with authentication
- **Profile System**: Organize parts and BOMs by user profiles

## Installation

### Prerequisites
- Rust 2024 edition or later
- Cargo

### Building from Source
```bash
# Clone the repository
git clone https://github.com/vincent-uden/pcb-parts-client
cd pcb-parts-client

# Build all components
cargo build --release

# Or build specific components
cargo build --bin gui --release
cargo build --bin cli --release
```

## Usage

### GUI Application
```bash
# Run with default configuration
cargo run --bin gui

# Run with custom configuration file
cargo run --bin gui -- --config path/to/config.conf
```

The GUI provides an intuitive interface for:
- Searching and browsing parts
- Managing stock levels
- Importing BOMs from CSV files
- Organizing parts in a visual grid layout
- User authentication and profile management

### CLI Application
```bash
# Create a new user
cargo run --bin cli -- create-user user@example.com password123

# Login
cargo run --bin cli -- login user@example.com password123

# List all parts
cargo run --bin cli -- list-parts

# Search parts by name
cargo run --bin cli -- list-parts --name "resistor"

# Add a new part
cargo run --bin cli -- add-part "10k Resistor" "1/4W 5% Carbon Film"

# Create a profile
cargo run --bin cli -- create-profile "My Workshop"

# Stock a part in a specific bin
cargo run --bin cli -- stock-part 1 42 100 5 3 0

# Import BOM from CSV
cargo run --bin cli -- add-bom 1 bom.csv "Project Alpha" "Main PCB" "Part Number" "Description" "Quantity"

# View BOM details
cargo run --bin cli -- show-bom 1 1
```
### Crates Overview

- **`gui`**: Desktop application built with Iced framework
- **`cli`**: Command-line tool for automation, scripting and testing
- **`common`**: Shared library containing data models, network client, and import utilities

## Configuration

### GUI Configuration
The GUI supports configuration files for customizing keybindings and settings:

```bash
cargo run --bin gui -- --config assets/default.conf
```

## Data Models

The application manages several key data types:

- **Parts**: Electronic components with names and descriptions
- **BOMs**: Bills of materials linking parts with quantities
- **Stock**: Inventory tracking with bin locations (3D coordinates)
- **Users & Profiles**: Multi-user organization system
- **Bins**: Physical storage locations

## Author

Vincent Udén
