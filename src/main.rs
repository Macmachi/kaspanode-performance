/*  
 * Author: Rymentz
 * Version: v1.0.0
 * License: MIT License
 */

 use std::{thread, time::Duration};
 use rusqlite::{Connection, Result};
 use sysinfo::{
     System,
     SystemExt,
     NetworksExt,
     NetworkExt,
     DiskExt,
     ProcessExt,
 };
 use std::time::{SystemTime, UNIX_EPOCH};
 use std::process::Command;
 use tui::{
     backend::CrosstermBackend,
     widgets::{Block, Borders, Chart, Dataset, GraphType, Paragraph, List, ListItem},
     layout::{Layout, Constraint, Direction, Alignment},
     text::{Span, Spans},
     style::{Style, Color},
     symbols,
     Terminal,
 };
 use crossterm::{
     event::{self, Event, KeyCode},
     execute,
     terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
 };
 use std::io::stdout;
 
 struct ServerMonitor {
     sys: System,
     cpu_history: Vec<(f64, f64)>,
     mem_history: Vec<(f64, f64)>,
     disk_history: Vec<(f64, f64)>,
     received_history: Vec<(f64, f64)>,
     transmitted_history: Vec<(f64, f64)>,
     ssh_attempts: Vec<(String, String, String)>,
     db: Connection,
     window_size: usize,
     log_scroll: usize,
     max_logs: usize,
     last_received: u64,
     last_transmitted: u64,
     last_network_time: SystemTime,
 }
 
 fn get_dir_size(path: &str) -> std::io::Result<u64> {
     let mut total_size = 0;
     for entry in std::fs::read_dir(path)? {
         let entry = entry?;
         let metadata = entry.metadata()?;
         if metadata.is_file() {
             total_size += metadata.len();
         } else if metadata.is_dir() {
             total_size += get_dir_size(entry.path().to_str().unwrap_or(""))?;
         }
     }
     Ok(total_size)
 }
 
 impl ServerMonitor {
    fn new() -> Result<Self> {
        let db = Connection::open("metrics.db")?;
    
        // System metrics table avec les nouvelles colonnes
        db.execute(
            "CREATE TABLE IF NOT EXISTS metrics (
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
            )",
            rusqlite::params![],
        )?;
 
         // SSH attempts table
         db.execute(
             "CREATE TABLE IF NOT EXISTS ssh_attempts (
                 timestamp INTEGER,
                 ip TEXT,
                 status TEXT,
                 PRIMARY KEY (timestamp, ip)
             )",
             rusqlite::params![],
         )?;
 
         // Indexes
         db.execute(
             "CREATE INDEX IF NOT EXISTS idx_timestamp ON metrics(timestamp)",
             rusqlite::params![],
         )?;
         db.execute(
             "CREATE INDEX IF NOT EXISTS idx_ssh_timestamp ON ssh_attempts(timestamp)",
             rusqlite::params![],
         )?;
 
         Ok(ServerMonitor {
             sys: System::new_all(),
             cpu_history: Vec::new(),
             mem_history: Vec::new(),
             disk_history: Vec::new(),
             received_history: Vec::new(),
             transmitted_history: Vec::new(),
             ssh_attempts: Vec::new(),
             db,
             window_size: 100,
             log_scroll: 0,
             max_logs: 1000, // Limit the number of logs kept in memory
             last_received: 0,
             last_transmitted: 0,
             last_network_time: SystemTime::now(),
         })
     }
 
     // Methods for scrolling logs
     fn scroll_logs_up(&mut self) {
         if self.log_scroll > 0 {
             self.log_scroll -= 1;
         }
     }
 
     fn scroll_logs_down(&mut self) {
         if self.log_scroll < self.ssh_attempts.len().saturating_sub(3) {
             self.log_scroll += 1;
         }
     }
 
     fn check_ssh_attempts(&mut self) -> Result<()> {
         let output = Command::new("journalctl")
             .args(["-u", "ssh", "--since", "1m", "-n", "50", "--no-pager"])
             .output()
             .expect("Failed to execute journalctl");
 
         let log = String::from_utf8_lossy(&output.stdout);
         for line in log.lines() {
             if line.contains("Failed password") || line.contains("Accepted password") {
                 let timestamp = SystemTime::now()
                     .duration_since(UNIX_EPOCH)
                     .unwrap()
                     .as_secs();
 
                 let ip = if let Some(ip) = line
                     .split("from ")
                     .nth(1)
                     .and_then(|s| s.split(' ').next())
                 {
                     ip.to_string()
                 } else {
                     "unknown".to_string()
                 };
 
                 let status = if line.contains("Failed") {
                     "Failed"
                 } else {
                     "Success"
                 };
 
                 self.db.execute(
                     "INSERT OR IGNORE INTO ssh_attempts (timestamp, ip, status)
                     VALUES (?1, ?2, ?3)",
                     (timestamp, &ip, status),
                 )?;
 
                 self.ssh_attempts.push((
                     timestamp.to_string(),
                     ip,
                     status.to_string(),
                 ));
             }
         }
 
         // Limit the number of logs in memory
         while self.ssh_attempts.len() > self.max_logs {
             self.ssh_attempts.remove(0);
         }
 
         Ok(())
     }
 
     fn log_to_db(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let timestamp = now
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs_f64();
            
        // Refresh all system information
        self.sys.refresh_all();
        
        // Attendre un peu pour des mesures plus précises
        thread::sleep(Duration::from_millis(200));
    
        // Rafraîchir à nouveau pour la mesure
        self.sys.refresh_cpu();
        self.sys.refresh_processes();
    
        // Get kaspad metrics with proper refresh
        let num_cores = self.sys.cpus().len() as f64;
        let (kaspad_cpu_usage, kaspad_memory, kaspad_disk_read, kaspad_disk_write) = 
            if let Some(process) = self.sys.processes_by_name("kaspad").next() {
                (
                    (process.cpu_usage() as f64) / num_cores,
                    process.memory() as f64 / 1_024_000.0, // Convert to GB
                    process.disk_usage().read_bytes as f64 / 1_024_000.0,
                    process.disk_usage().written_bytes as f64 / 1_024_000.0
                )
            } else {
                (0.0, 0.0, 0.0, 0.0)
            };
    
        // Update CPU history
        self.cpu_history.push((timestamp, kaspad_cpu_usage));
    
        // Calculate memory percentage
        let total_memory_gb = self.sys.total_memory() as f64 / 1_024_000.0;
        let memory_usage = (kaspad_memory / total_memory_gb) * 100.0;
        self.mem_history.push((timestamp as f64, memory_usage));
    
        // Network calculations
        let total_received = self
            .sys
            .networks()
            .iter()
            .map(|(_, data)| data.received())
            .sum::<u64>();
        let total_transmitted = self
            .sys
            .networks()
            .iter()
            .map(|(_, data)| data.transmitted())
            .sum::<u64>();
    
        let time_diff = now
            .duration_since(self.last_network_time)
            .unwrap_or(Duration::from_secs(1))
            .as_secs_f64();
    
        if time_diff > 0.0 {
            let received_speed = if self.last_received > 0 && total_received >= self.last_received {
                (total_received - self.last_received) as f64 / (time_diff * 1_048_576.0)
            } else {
                0.0
            };
    
            let transmitted_speed =
                if self.last_transmitted > 0 && total_transmitted >= self.last_transmitted {
                    (total_transmitted - self.last_transmitted) as f64 / (time_diff * 1_048_576.0)
                } else {
                    0.0
                };
    
            self.received_history.push((timestamp, received_speed));
            self.transmitted_history.push((timestamp, transmitted_speed));
    
            self.last_received = total_received;
            self.last_transmitted = total_transmitted;
            self.last_network_time = now;
        } else {
            self.received_history.push((timestamp, 0.0));
            self.transmitted_history.push((timestamp, 0.0));
        }
    
        // Disk usage including kaspad directory
        let disk_usage = if let Some(disk) = self.sys.disks().into_iter().next() {
            let home = std::env::var("HOME").unwrap_or_else(|_| String::from("/root"));
            let kaspa_dir = format!("{}/.kaspa", home);
    
            if let Ok(size) = get_dir_size(&kaspa_dir) {
                let total_space = disk.total_space();
                let free_space = disk.available_space();
                let used_space = total_space - free_space + size;
                (used_space as f64 / total_space as f64) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };
    
        self.disk_history.push((timestamp as f64, disk_usage));
    
        // Maintain window size for all histories
        if self.cpu_history.len() > self.window_size {
            self.cpu_history.remove(0);
        }
        if self.mem_history.len() > self.window_size {
            self.mem_history.remove(0);
        }
        if self.disk_history.len() > self.window_size {
            self.disk_history.remove(0);
        }
        if self.received_history.len() > self.window_size {
            self.received_history.remove(0);
        }
        if self.transmitted_history.len() > self.window_size {
            self.transmitted_history.remove(0);
        }
    
        // Save all metrics to database
        self.db.execute(
            "INSERT INTO metrics (
                timestamp, cpu_usage, memory_usage, memory_total,
                memory_used, disk_usage, network_received, network_transmitted,
                kaspad_memory, kaspad_disk_read, kaspad_disk_write
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                timestamp as i64,
                kaspad_cpu_usage,
                memory_usage as f64,
                self.sys.total_memory() as i64,
                (kaspad_memory * 1_024_000.0) as i64,
                disk_usage,
                total_received as i64,
                total_transmitted as i64,
                (kaspad_memory * 1_024_000.0) as i64,
                (kaspad_disk_read * 1_024_000.0) as i64,
                (kaspad_disk_write * 1_024_000.0) as i64,
            ],
        )?;
    
        Ok(())
    }
 
     fn update(&mut self) -> Result<()> {
         self.log_to_db()?;
         self.check_ssh_attempts()?;
         Ok(())
     }
 
     fn cleanup(&self) -> Result<()> {
         self.db.execute("VACUUM", rusqlite::params![])?;
         self.db.execute("ANALYZE", rusqlite::params![])?;
         Ok(())
     }
 
     fn draw<B: tui::backend::Backend>(&mut self, terminal: &mut Terminal<B>) -> std::io::Result<()> {
         terminal.draw(|f| {
             let chunks = Layout::default()
                 .direction(Direction::Vertical)
                 .constraints(
                     [
                         Constraint::Percentage(20), // CPU
                         Constraint::Percentage(20), // Memory
                         Constraint::Percentage(20), // Disk
                         Constraint::Percentage(20), // Network
                         Constraint::Percentage(15), // Logs
                         Constraint::Percentage(5),  // Author info
                     ]
                     .as_ref(),
                 )
                 .split(f.size());
 
            // CPU Graph
            let current_cpu = self.cpu_history.last().map(|&(_, v)| v).unwrap_or(0.0);
            let num_cores = self.sys.cpus().len();

            // New format for the CPU title that includes real-time usage and core count
            let cpu_title = format!("kaspad CPU Usage ({:.1}%) - {} Cores", current_cpu, num_cores);
            let cpu_label = format!("CPU: {:.1}% of {} Cores", current_cpu, num_cores);

            let cpu_dataset = Dataset::default()
                .name(cpu_label.as_str())
                 .marker(symbols::Marker::Dot)
                 .graph_type(GraphType::Line)
                 .style(Style::default().fg(Color::Cyan))
                 .data(&self.cpu_history);
 
             let cpu_chart = Chart::new(vec![cpu_dataset])
                 .block(Block::default().title(cpu_title).borders(Borders::ALL))
                 .x_axis(tui::widgets::Axis::default().bounds([
                     self.cpu_history.first().map(|p| p.0).unwrap_or(0.0),
                     self.cpu_history.last().map(|p| p.0).unwrap_or(100.0),
                 ]))
                 .y_axis(tui::widgets::Axis::default().bounds([0.0, 100.0]));
 
             f.render_widget(cpu_chart, chunks[0]);
 
             // Memory Graph
             let current_mem = self.mem_history.last().map(|&(_, v)| v).unwrap_or(0.0);
             let total_mem = self.sys.total_memory() as f64 / 1_024_000.0; // Convert to GB
             let used_mem = total_mem * current_mem / 100.0;
             let mem_label = format!(
                 "MEM: {:.1}GB / {:.1}GB ({:.1}%)",
                 used_mem, total_mem, current_mem
             );
             let mem_title = format!(
                 "Memory Usage ({:.1}GB of {:.1}GB)",
                 used_mem, total_mem
             );
 
             let mem_dataset = Dataset::default()
                 .name(mem_label.as_str())
                 .marker(symbols::Marker::Dot)
                 .graph_type(GraphType::Line)
                 .style(Style::default().fg(Color::Green))
                 .data(&self.mem_history);
 
             let mem_chart = Chart::new(vec![mem_dataset])
                 .block(Block::default().title(mem_title.as_str()).borders(Borders::ALL))
                 .x_axis(tui::widgets::Axis::default().bounds([
                     self.mem_history.first().map(|p| p.0).unwrap_or(0.0),
                     self.mem_history.last().map(|p| p.0).unwrap_or(100.0),
                 ]))
                 .y_axis(tui::widgets::Axis::default().bounds([0.0, 100.0]));
 
             f.render_widget(mem_chart, chunks[1]);
 
             // Disk Graph
             let current_disk = self.disk_history.last().map(|&(_, v)| v).unwrap_or(0.0);
             let disk_info = self
                 .sys
                 .disks()
                 .into_iter()
                 .next()
                 .map(|disk| {
                     let total = disk.total_space() as f64 / 1_000_000_000.0; // Convert to GB
                     let free = disk.available_space() as f64 / 1_000_000_000.0;
                     let used = total - free;
                     (total, used)
                 })
                 .unwrap_or((0.0, 0.0));
 
             let disk_label = format!(
                 "Disk: {:.1}GB used / {:.1}GB total ({:.1}%)",
                 disk_info.1, disk_info.0, current_disk
             );
 
             let disk_title = format!(
                 "Disk Usage ({:.1}GB of {:.1}GB)",
                 disk_info.1, disk_info.0
             );
 
             let disk_dataset = Dataset::default()
                 .name(disk_label.as_str())
                 .marker(symbols::Marker::Dot)
                 .graph_type(GraphType::Line)
                 .style(Style::default().fg(Color::Yellow))
                 .data(&self.disk_history);
 
             let disk_chart = Chart::new(vec![disk_dataset])
                 .block(Block::default().title(disk_title.as_str()).borders(Borders::ALL))
                 .x_axis(tui::widgets::Axis::default().bounds([
                     self.disk_history.first().map(|p| p.0).unwrap_or(0.0),
                     self.disk_history.last().map(|p| p.0).unwrap_or(100.0),
                 ]))
                 .y_axis(tui::widgets::Axis::default().bounds([0.0, 100.0]));
 
             f.render_widget(disk_chart, chunks[2]);
 
             // Network Graph
             // Get the latest network speed values
             let current_received = self.received_history.last().map(|&(_, v)| v).unwrap_or(0.0);
             let current_transmitted = self.transmitted_history.last().map(|&(_, v)| v).unwrap_or(0.0);
 
             // Create the label with actual speeds
             let net_label = format!(
                 "↓ {:.2} MB/s, ↑ {:.2} MB/s",
                 current_received, current_transmitted
             );
 
             // Use variables in the graph title
             let net_title = format!("Network Traffic ({})", net_label);
 
             // Create datasets for download and upload
             let received_dataset = Dataset::default()
                 .name("Download")
                 .marker(symbols::Marker::Dot)
                 .graph_type(GraphType::Line)
                 .style(Style::default().fg(Color::Blue))
                 .data(&self.received_history);
 
             let transmitted_dataset = Dataset::default()
                 .name("Upload")
                 .marker(symbols::Marker::Dot)
                 .graph_type(GraphType::Line)
                 .style(Style::default().fg(Color::Magenta))
                 .data(&self.transmitted_history);
 
             // Create the chart with both datasets
             let net_chart = Chart::new(vec![received_dataset, transmitted_dataset])
                 .block(Block::default().title(net_title).borders(Borders::ALL))
                 .x_axis(tui::widgets::Axis::default().bounds([
                     self.received_history.first().map(|p| p.0).unwrap_or(0.0),
                     self.received_history.last().map(|p| p.0).unwrap_or(100.0),
                 ]))
                 .y_axis(
                     tui::widgets::Axis::default().bounds([
                         0.0,
                         self.received_history
                             .iter()
                             .chain(self.transmitted_history.iter())
                             .map(|p| p.1)
                             .fold(0.0, f64::max),
                     ]),
                 );
 
             f.render_widget(net_chart, chunks[3]);
 
             // Logs section
             let log_block = Block::default()
                 .title("System Logs (↑↓ to scroll)")
                 .borders(Borders::ALL);
 
             let logs: Vec<ListItem> = self
                 .ssh_attempts
                 .iter()
                 .rev()
                 .skip(self.log_scroll)
                 .take(3)
                 .map(|(timestamp, ip, status)| {
                     let time = SystemTime::now()
                         .duration_since(UNIX_EPOCH)
                         .unwrap()
                         .as_secs() as i64
                         - timestamp.parse::<i64>().unwrap_or(0);
 
                     let time_str = if time < 60 {
                         format!("{}s ago", time)
                     } else if time < 3600 {
                         format!("{}m ago", time / 60)
                     } else {
                         format!("{}h ago", time / 3600)
                     };
 
                     ListItem::new(Spans::from(vec![
                         Span::styled(
                             format!("[{}] ", time_str),
                             Style::default().fg(Color::Gray),
                         ),
                         Span::styled(
                             format!("{}: {}", ip, status),
                             Style::default().fg(if status == "Failed" {
                                 Color::Red
                             } else {
                                 Color::Green
                             }),
                         ),
                     ]))
                 })
                 .collect();
 
             let log_list = List::new(logs)
                 .block(log_block)
                 .style(Style::default().fg(Color::White));
 
             f.render_widget(log_list, chunks[4]);
 
             // Author section
             let info_block = Block::default().borders(Borders::ALL);
 
             let info_text = Paragraph::new("Rymentz - kaspa:qqngpnpwrfhexgu8kzk3lteu5fakh6fylmt53gt7qwtf4vttjyvfyrnr8shwa")
                 .style(Style::default().fg(Color::White))
                 .block(info_block)
                 .alignment(Alignment::Center);
 
             f.render_widget(info_text, chunks[5]);
         })?;
         Ok(())
     }
 }
 
 impl From<rusqlite::Error> for Error {
     fn from(_: rusqlite::Error) -> Error {
         Error::Rusqlite(())
     }
 }
 
 impl From<std::io::Error> for Error {
     fn from(_: std::io::Error) -> Error {
         Error::Io(())
     }
 }
 
 impl From<std::string::FromUtf8Error> for Error {
     fn from(_: std::string::FromUtf8Error) -> Error {
         Error::Utf8(())
     }
 }
 
 impl From<std::time::SystemTimeError> for Error {
     fn from(_: std::time::SystemTimeError) -> Error {
         Error::SystemTime(())
     }
 }
 
 #[derive(Debug)]
 enum Error {
     Rusqlite(()),
     Io(()),
     Utf8(()),
     SystemTime(()),
 }
 
 fn main() -> Result<(), Error> {
     enable_raw_mode()?;
     let mut stdout = stdout();
     execute!(stdout, EnterAlternateScreen)?;
     let backend = CrosstermBackend::new(stdout);
     let mut terminal = Terminal::new(backend)?;
 
     let mut monitor = ServerMonitor::new()?;
     let mut cleanup_counter = 0;
     let mut last_update = SystemTime::now();
 
     loop {
         // Event handling with timeout
         if event::poll(Duration::from_millis(250))? {
             if let Event::Key(key) = event::read()? {
                 match key.code {
                     KeyCode::Char('q') => break,
                     KeyCode::Up => monitor.scroll_logs_up(),
                     KeyCode::Down => monitor.scroll_logs_down(),
                     _ => {}
                 }
             }
         }
 
         // Periodic update every 2 seconds
         if SystemTime::now().duration_since(last_update)?.as_secs() >= 2 {
             monitor.update()?;
             last_update = SystemTime::now();
 
             cleanup_counter += 1;
             if cleanup_counter >= 21600 {
                 monitor.cleanup()?;
                 cleanup_counter = 0;
             }
         }
 
         // Render the interface
         if let Err(e) = monitor.draw(&mut terminal) {
             eprintln!("Draw error: {}", e);
         }
 
         // Pause to reduce CPU usage
         thread::sleep(Duration::from_millis(250));
     }
 
     // Cleanup
     disable_raw_mode()?;
     execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
     terminal.show_cursor()?;
 
     Ok(())
 }