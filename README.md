# Server Monitor

A real-time system monitor with TUI (Terminal User Interface) for tracking your Kaspa node performance.

## 🚀 Features

- Real-time CPU monitoring for Kaspad
- Memory usage tracking for Kaspad
- Disk space monitoring for Kaspad
- Network monitoring (upload/download rates) for Kaspad
- SSH connection attempts logging
- SQLite database for metrics history
- Interactive terminal user interface

## 📋 Prerequisites

- Rust  
- SQLite3
- Linux  
- Running rusty kaspad node

## ⚡ Important Note

This monitor requires a running kaspad instance to track its performance metrics. Make sure to:

1. Install kaspad first

## 🛠️ Installation

1. Install Rust and Cargo
2. Install system dependencies 

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
