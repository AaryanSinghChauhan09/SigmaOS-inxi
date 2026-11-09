// SPDX-License-Identifier: MIT
// SigmaOS-inxi - System Information Tool component
// inxi-style comprehensive system info reporter for SigmaOS,
// inspired by Arch's inxi package.

use std::collections::HashMap;
use std::fmt;

// ============================================================================
// SYSTEM INFO
// ============================================================================

/// CPU information
#[derive(Debug, Clone, Default)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
    pub mhz: u32,
    pub cache_l1: u32,   // KB
    pub cache_l2: u32,   // KB
    pub cache_l3: u32,   // MB
    pub flags: Vec<String>,
    pub virtualization: bool,
    pub governor: String,
}

impl CpuInfo {
    pub fn cpu_count(&self) -> u32 { self.cores }
    pub fn thread_count(&self) -> u32 { self.threads }
    pub fn has_flag(&self, flag: &str) -> bool {
        self.flags.iter().any(|f| f == flag)
    }
    pub fn supports(&self, feature: &str) -> bool {
        self.has_flag(feature)
    }
}

/// Memory information
#[derive(Debug, Clone, Default)]
pub struct MemInfo {
    pub total_kb: u64,
    pub used_kb: u64,
    pub free_kb: u64,
    pub buffers_kb: u64,
    pub cached_kb: u64,
    pub swap_total_kb: u64,
    pub swap_used_kb: u64,
    pub swap_free_kb: u64,
}

impl MemInfo {
    pub fn total_mb(&self) -> u64 { self.total_kb / 1024 }
    pub fn used_mb(&self) -> u64 { self.used_kb / 1024 }
    pub fn free_mb(&self) -> u64 { self.free_kb / 1024 }
    pub fn usage_percent(&self) -> f64 {
        if self.total_kb == 0 { return 0.0; }
        self.used_kb as f64 / self.total_kb as f64 * 100.0
    }
    pub fn swap_usage_percent(&self) -> f64 {
        if self.swap_total_kb == 0 { return 0.0; }
        self.swap_used_kb as f64 / self.swap_total_kb as f64 * 100.0
    }
}

/// Disk/partition info
#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub device: String,
    pub model: String,
    pub size_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub filesystem: String,
    pub mountpoint: String,
    pub rotational: bool,  // HDD vs SSD
}

impl DiskInfo {
    pub fn usage_percent(&self) -> f64 {
        if self.size_bytes == 0 { return 0.0; }
        self.used_bytes as f64 / self.size_bytes as f64 * 100.0
    }
    pub fn is_ssd(&self) -> bool { !self.rotational }
}

/// GPU info
#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub vendor: String,    // "NVIDIA", "AMD", "Intel"
    pub model: String,
    pub driver: String,
    pub vram_mb: u32,
    pub pci_id: String,
}

/// Network interface info
#[derive(Debug, Clone)]
pub struct NetInfo {
    pub name: String,
    pub mac: String,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub state: NetState,
    pub speed_mbps: Option<u32>,
    pub is_wireless: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetState {
    Up,
    Down,
    Unknown,
}

/// Battery info
#[derive(Debug, Clone)]
pub struct BatteryInfo {
    pub present: bool,
    pub capacity_percent: u8,
    pub status: BatteryStatus,
    pub time_remaining_min: Option<u32>,
    pub model: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

/// OS/Kernel info
#[derive(Debug, Clone, Default)]
pub struct OsInfo {
    pub name: String,        // "SigmaOS"
    pub version: String,
    pub kernel: String,
    pub kernel_release: String,
    pub arch: String,
    pub hostname: String,
    pub uptime_seconds: u64,
    pub boot_time_ms: u64,
}

/// Display server info
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub server: String,      // "Wayland", "X11"
    pub compositor: String,  // "Hyprland", "GNOME", "KDE"
    pub resolution: String,
    pub refresh_hz: u32,
}

// ============================================================================
// SYSTEM REPORT
// ============================================================================

/// Complete system information report
#[derive(Debug, Clone, Default)]
pub struct SystemReport {
    pub cpu: CpuInfo,
    pub mem: MemInfo,
    pub disks: Vec<DiskInfo>,
    pub gpus: Vec<GpuInfo>,
    pub network: Vec<NetInfo>,
    pub battery: Option<BatteryInfo>,
    pub os: OsInfo,
    pub display: Option<DisplayInfo>,
}

impl SystemReport {
    /// Collect system info (simulated — real impl reads /proc, /sys, etc.)
    pub fn collect() -> Self {
        Self {
            cpu: CpuInfo::default(),
            mem: MemInfo::default(),
            disks: Vec::new(),
            gpus: Vec::new(),
            network: Vec::new(),
            battery: None,
            os: OsInfo {
                name: "SigmaOS".into(),
                version: "0.1.0".into(),
                kernel: "sigma-kernel".into(),
                kernel_release: "6.1.0-sigma".into(),
                arch: "x86_64".into(),
                hostname: "sigmaos".into(),
                uptime_seconds: 0,
                boot_time_ms: 0,
            },
            display: None,
        }
    }

    /// Format as inxi-style output
    pub fn to_inxi(&self, verbose: bool) -> String {
        let mut out = String::new();

        // System line
        out.push_str(&format!("System: {} {} Kernel: {} {} Arch: {}\n",
            self.os.name, self.os.version, self.os.kernel, self.os.kernel_release, self.os.arch));

        // CPU line
        out.push_str(&format!("CPU: {} ({} cores, {} threads) @ {} MHz\n",
            self.cpu.model, self.cpu.cores, self.cpu.threads, self.cpu.mhz));

        // Memory line
        out.push_str(&format!("Memory: {}MiB used / {}MiB total ({:.1}%)\n",
            self.mem.used_mb(), self.mem.total_mb(), self.mem.usage_percent()));
        if self.mem.swap_total_kb > 0 {
            out.push_str(&format!("Swap: {}MiB used / {}MiB total ({:.1}%)\n",
                self.mem.swap_used_kb / 1024, self.mem.swap_total_kb / 1024,
                self.mem.swap_usage_percent()));
        }

        // GPU
        for gpu in &self.gpus {
            out.push_str(&format!("GPU: {} {} ({} VRAM, driver: {})\n",
                gpu.vendor, gpu.model, gpu.vram_mb, gpu.driver));
        }

        // Disks
        for disk in &self.disks {
            let type_str = if disk.is_ssd() { "SSD" } else { "HDD" };
            out.push_str(&format!("Disk: {} {} {} ({:.1}% used, {})\n",
                disk.device, disk.model, type_str, disk.usage_percent(), disk.filesystem));
        }

        // Network
        for net in &self.network {
            let state = match net.state {
                NetState::Up => "UP",
                NetState::Down => "DOWN",
                NetState::Unknown => "UNKNOWN",
            };
            out.push_str(&format!("Net: {} {} {} {}\n",
                net.name, state, net.ipv4.as_deref().unwrap_or("-"),
                if net.is_wireless { "WiFi" } else { "Eth" }));
        }

        // Battery
        if let Some(bat) = &self.battery {
            if bat.present {
                let status = match bat.status {
                    BatteryStatus::Charging => "Charging",
                    BatteryStatus::Discharging => "Discharging",
                    BatteryStatus::Full => "Full",
                    BatteryStatus::Unknown => "Unknown",
                };
                out.push_str(&format!("Battery: {}% {} ({})\n",
                    bat.capacity_percent, status, bat.model));
            }
        }

        // Display
        if let Some(d) = &self.display {
            out.push_str(&format!("Display: {} {} {} @ {}Hz\n",
                d.server, d.compositor, d.resolution, d.refresh_hz));
        }

        // Uptime
        let hours = self.os.uptime_seconds / 3600;
        let mins = (self.os.uptime_seconds % 3600) / 60;
        out.push_str(&format!("Uptime: {}h {}m\n", hours, mins));

        if verbose {
            // CPU flags
            if !self.cpu.flags.is_empty() {
                out.push_str(&format!("CPU Flags: {}\n", self.cpu.flags.join(" ")));
            }
            // Cache
            out.push_str(&format!("Cache: L1={}KB L2={}KB L3={}MB\n",
                self.cpu.cache_l1, self.cpu.cache_l2, self.cpu.cache_l3));
        }

        out
    }
}

// ============================================================================
// ERRORS
// ============================================================================

#[derive(Debug)]
pub enum InxiError {
    Io(String),
    Parse(String),
}

impl fmt::Display for InxiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(m) => write!(f, "io error: {}", m),
            Self::Parse(m) => write!(f, "parse error: {}", m),
        }
    }
}
impl std::error::Error for InxiError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mem_info() {
        let mem = MemInfo {
            total_kb: 16_000_000,
            used_kb: 8_000_000,
            free_kb: 8_000_000,
            buffers_kb: 0, cached_kb: 0,
            swap_total_kb: 4_000_000, swap_used_kb: 1_000_000, swap_free_kb: 3_000_000,
        };
        assert_eq!(mem.total_mb(), 15625);
        assert!((mem.usage_percent() - 50.0).abs() < 0.1);
        assert!((mem.swap_usage_percent() - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_cpu_info() {
        let cpu = CpuInfo {
            model: "AMD Ryzen 9 7950X".into(),
            cores: 16, threads: 32, mhz: 4200,
            flags: vec!["avx2".into(), "sse4_2".into()],
            ..Default::default()
        };
        assert_eq!(cpu.cpu_count(), 16);
        assert_eq!(cpu.thread_count(), 32);
        assert!(cpu.has_flag("avx2"));
        assert!(!cpu.has_flag("avx512"));
    }

    #[test]
    fn test_disk_info() {
        let disk = DiskInfo {
            device: "/dev/nvme0n1".into(), model: "Samsung 980 Pro".into(),
            size_bytes: 2_000_000_000_000, used_bytes: 1_000_000_000_000,
            free_bytes: 1_000_000_000_000, filesystem: "btrfs".into(),
            mountpoint: "/".into(), rotational: false,
        };
        assert!(disk.is_ssd());
        assert!((disk.usage_percent() - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_system_report() {
        let mut report = SystemReport::collect();
        report.cpu.model = "Test CPU".into();
        report.cpu.cores = 8;
        report.mem.total_kb = 8_000_000;
        report.mem.used_kb = 4_000_000;
        report.mem.free_kb = 4_000_000;
        report.gpus.push(GpuInfo {
            vendor: "NVIDIA".into(), model: "RTX 4090".into(),
            driver: "nvidia".into(), vram_mb: 24576, pci_id: "10de:2684".into(),
        });
        report.network.push(NetInfo {
            name: "eth0".into(), mac: "aa:bb:cc:dd:ee:ff".into(),
            ipv4: Some("192.168.1.100".into()), ipv6: None,
            state: NetState::Up, speed_mbps: Some(1000), is_wireless: false,
        });
        report.battery = Some(BatteryInfo {
            present: true, capacity_percent: 85,
            status: BatteryStatus::Discharging,
            time_remaining_min: Some(180), model: "BAT0".into(),
        });

        let out = report.to_inxi(false);
        assert!(out.contains("SigmaOS"));
        assert!(out.contains("Test CPU"));
        assert!(out.contains("NVIDIA"));
        assert!(out.contains("eth0"));
        assert!(out.contains("85%"));
    }

    #[test]
    fn test_verbose_output() {
        let mut report = SystemReport::collect();
        report.cpu.flags = vec!["avx2".into(), "aes".into()];
        report.cpu.cache_l1 = 512;
        let out = report.to_inxi(true);
        assert!(out.contains("CPU Flags"));
        assert!(out.contains("avx2"));
        assert!(out.contains("Cache"));
    }

    #[test]
    fn test_no_battery() {
        let report = SystemReport::collect();
        let out = report.to_inxi(false);
        assert!(!out.contains("Battery"));
    }
}
