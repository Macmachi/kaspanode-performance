# Kaspa Node Monitor

A real-time system monitor with TUI (Terminal User Interface) to track your Kaspa node performance.

![Demo](/images/demo.png)

## ⭐ Features

- Real-time monitoring of kaspad process:
  - CPU usage per core
  - Memory consumption
  - Disk usage
  - Network traffic (upload/download rates)
- SSH connection attempts logging
- SQLite database for metrics history
- Interactive terminal user interface with graphs
- Automatic data updates every 2 seconds
- Automatic database cleanup

## 📋 Prerequisites

- Rust and Cargo
- SQLite3
- Linux
- Running kaspad node

## 🛠️ Installation

1. Clone the repository:
```bash
git clone [repo-url]
cd server-monitor
```

2. Build the project:
```bash
cargo build --release
```

## 🚀 Usage

Start the monitor:
```bash
./target/release/server_monitor
```

### Available Commands

- `q` : Quit application
- `↑` : Scroll logs up
- `↓` : Scroll logs down

## 📊 Monitored Metrics

- **CPU**: Percentage used by kaspad
- **Memory**: Usage in GB and percentage
- **Disk**: 
  - Space used by .kaspa directory
  - Kaspad process reads/writes
- **Network**: 
  - Download rate
  - Upload rate
- **SSH**: Connection attempts (successful/failed)

## 🗃️ Database

The program uses SQLite to store metrics in `metrics.db`:

### Tables
- `metrics`: Timestamped system metrics
- `ssh_attempts`: SSH attempts history

### Data Structure
```sql
CREATE TABLE metrics (
    timestamp INTEGER PRIMARY KEY,
    cpu_usage REAL,
    memory_usage REAL,
    memory_total INTEGER,
    memory_used INTEGER,
    disk_usage REAL,
    network_received INTEGER,
    network_transmitted INTEGER,
    kaspad_memory INTEGER,
    kaspad_disk_read INTEGER,
    kaspad_disk_write INTEGER
);

CREATE TABLE ssh_attempts (
    timestamp INTEGER,
    ip TEXT,
    status TEXT,
    PRIMARY KEY (timestamp, ip)
);
```

## ⚙️ Configuration

Parameters are defined in code:
- `window_size`: Number of points in graphs (default: 100)
- `max_logs`: Maximum SSH logs in memory (default: 1000)
- Update interval: 2 seconds
- Database cleanup: every week

## ✍️ Author

Rymentz

## 💝 Support

If you find this tool useful, you can donate Kaspa to:
```
kaspa:qqngpnpwrfhexgu8kzk3lteu5fakh6fylmt53gt7qwtf4vttjyvfyrnr8shwa
```

## 🐛 Troubleshooting

1. No kaspad metrics displayed:
   - Check if kaspad is running: `ps aux | grep kaspad`
   - Check kaspad logs: `journalctl -u kaspad -f`

2. Database errors:
   - Check permissions: `ls -l metrics.db`
   - Verify SQLite installation: `sqlite3 --version`

3. Network monitoring issues:
   - Check user permissions
   - Run with sudo if needed (not recommended)
