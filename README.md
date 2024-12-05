```markdown
# Server Monitor

A real-time system monitor with TUI (Terminal User Interface) for tracking your Kaspa server performance.

## 🚀 Features

- Real-time CPU monitoring
- Memory usage tracking
- Disk space monitoring
- Network monitoring (upload/download rates)
- SSH connection attempts logging
- SQLite database for metrics history
- Interactive terminal user interface

## 📋 Prerequisites

- Rust (2021 edition)
- SQLite3
- Linux (for journctl features)
- Running kaspad instance

## ⚡ Important Note

This monitor requires a running kaspad instance to track its performance metrics. Make sure to:

1. Install kaspad first:
```bash
git clone https://github.com/kaspanet/kaspad.git
cd kaspad
go install .
```

2. Run kaspad in a separate terminal or as a service:
```bash
kaspad --utxoindex --appdir=~/.kaspa
```

## 🛠️ Installation

1. Install Rust and Cargo:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install system dependencies:
```bash
# Debian/Ubuntu
sudo apt install sqlite3 libsqlite3-dev pkg-config

# Fedora
sudo dnf install sqlite-devel pkgconfig

# Arch Linux
sudo pacman -S sqlite pkg-config
```

3. Clone the repository:
```bash
git clone https://github.com/your-username/server-monitor.git
cd server-monitor
```

4. Build and install:
```bash
cargo build --release
```

## 🚦 Usage

Start the monitor:
```bash
./target/release/server_monitor
```

### Available Commands:

- `q` : Quit application
- `↑` : Scroll up in logs
- `↓` : Scroll down in logs

## 📦 File Structure

```
server_monitor/
├── src/
│   └── main.rs
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## 🗃️ Database

The program uses SQLite to store metrics. The database (`metrics.db`) is automatically created with two tables:

- `metrics` : Stores system metrics
- `ssh_attempts` : Records SSH connection attempts

## 🔧 Configuration

Configuration is done through code constants:

- `window_size` : Number of points in graphs (default: 100)
- `max_logs` : Maximum number of SSH logs in memory (default: 1000)

## 🔍 Monitoring Details

The monitor tracks the following kaspad metrics:
- CPU usage per core
- Memory consumption
- Disk usage for the .kaspa directory
- Network traffic related to kaspad
- System-wide performance metrics

## 🚀 Running as a Service

To run both kaspad and the monitor as services:

1. Create kaspad service file:
```bash
sudo nano /etc/systemd/system/kaspad.service
```

```ini
[Unit]
Description=Kaspad Node
After=network.target

[Service]
User=your_username
ExecStart=/usr/local/bin/kaspad --utxoindex --appdir=/home/your_username/.kaspa
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

2. Create monitor service file:
```bash
sudo nano /etc/systemd/system/server-monitor.service
```

```ini
[Unit]
Description=Server Monitor
After=kaspad.service

[Service]
User=your_username
ExecStart=/path/to/server_monitor
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

3. Enable and start services:
```bash
sudo systemctl enable kaspad
sudo systemctl enable server-monitor
sudo systemctl start kaspad
sudo systemctl start server-monitor
```

## 🤝 Contributing

Contributions are welcome! Feel free to:

1. Fork the project
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ✍️ Author

Rymentz

## 🙏 Acknowledgments

If you find this project useful, you can donate Kaspa to:
`kaspa:qqngpnpwrfhexgu8kzk3lteu5fakh6fylmt53gt7qwtf4vttjyvfyrnr8shwa`

## 🐛 Troubleshooting

Common issues and solutions:

1. Monitor shows no kaspad metrics:
   - Verify kaspad is running: `ps aux | grep kaspad`
   - Check kaspad logs: `journalctl -u kaspad -f`

2. Database errors:
   - Check permissions: `ls -l metrics.db`
   - Ensure SQLite is installed: `sqlite3 --version`

3. Network monitoring issues:
   - Verify user permissions for network interfaces
   - Run with sudo if needed (not recommended for regular use)
```

Would you like me to explain any specific section in more detail or add additional information?
