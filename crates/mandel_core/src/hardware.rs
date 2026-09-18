//! 硬件信息采集：读取宿主 DMI/SMBIOS（对应虚拟机 HAAVK 硬件铭牌）。
//!
//! - Linux：读 `/sys/class/dmi/id/*` + `/proc/cpuinfo` + `/proc/meminfo`
//! - Windows：调用 PowerShell `Get-CimInstance` 读取 SMBIOS/WMI
//! - 其他平台：返回 unknown
//!
//! 曼德尔核心只**读取展示**硬件信息，不接管硬件。

use log::{debug, info};
use serde::Serialize;

use crate::config::MandelConfig;

#[derive(Debug, Clone, Default, Serialize)]
pub struct HardwareInfo {
    pub bios_vendor: String,
    pub manufacturer: String,
    pub product_name: String,
    pub serial: String,
    pub cpu: String,
    pub memory_mb: u64,
    pub network: String,
}

impl HardwareInfo {
    /// 平台无关的空信息（无法读取时使用）
    pub fn unknown() -> Self {
        Self {
            bios_vendor: "HAAVK Group".into(),
            manufacturer: "HAAVK哈夫克集团".into(),
            product_name: "HAAVK全域算力终端".into(),
            serial: "HAAVK-NODE-UNKNOWN".into(),
            cpu: "Mandel CPU (未知)".into(),
            memory_mb: 0,
            network: "e1000 (未知)".into(),
        }
    }

    /// 汇总为一行摘要
    pub fn summary(&self) -> String {
        format!(
            "{} | CPU: {} | RAM: {} MB | 网卡: {} | SN: {}",
            self.product_name, self.cpu, self.memory_mb, self.network, self.serial
        )
    }
}

/// 采集宿主硬件信息（受 `read_dmi` 开关控制）
pub fn collect(config: &MandelConfig) -> HardwareInfo {
    let mut info = collect_impl();
    if info.memory_mb == 0 {
        info.memory_mb = config.hardware.memory_limit_mb;
    }
    if info.network.is_empty() {
        info.network = "e1000".into();
    }
    debug!("DMI 硬件信息：{}", info.summary());
    info
}

#[cfg(target_os = "linux")]
fn collect_impl() -> HardwareInfo {
    let read = |p: &str| -> String {
        std::fs::read_to_string(p).map(|s| s.trim().to_string()).unwrap_or_default()
    };

    let bios_vendor = read("/sys/class/dmi/id/bios_vendor");
    let manufacturer = read("/sys/class/dmi/id/sys_vendor");
    let product_name = read("/sys/class/dmi/id/product_name");
    let serial = read("/sys/class/dmi/id/board_serial");

    let cpu = read("/proc/cpuinfo")
        .lines()
        .find_map(|l| {
            if l.trim_start().starts_with("model name") {
                l.splitn(2, ':').nth(1).map(|v| v.trim().to_string())
            } else {
                None
            }
        })
        .unwrap_or_else(|| "Mandel CPU".into());

    let memory_mb = read("/proc/meminfo")
        .lines()
        .find_map(|l| l.strip_prefix("MemTotal"))
        .and_then(|v| v.trim_start_matches(':').trim().trim_end_matches("kB").trim().parse::<u64>().ok())
        .map(|kb| kb / 1024)
        .unwrap_or(0);

    let network = std::fs::read_dir("/sys/class/net")
        .map(|it| {
            it.filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .find(|n| n != "lo")
                .unwrap_or_else(|| "e1000".into())
        })
        .unwrap_or_else(|_| "e1000".into());

    info!("读取 Linux 宿主 DMI/SMBIOS 成功");

    HardwareInfo {
        bios_vendor: nonempty(&bios_vendor, "HAAVK Group"),
        manufacturer: nonempty(&manufacturer, "HAAVK哈夫克集团"),
        product_name: nonempty(&product_name, "HAAVK全域算力终端"),
        serial: nonempty(&serial, "HAAVK-NODE-UNKNOWN"),
        cpu,
        memory_mb,
        network,
    }
}

#[cfg(target_os = "windows")]
fn collect_impl() -> HardwareInfo {
    use std::process::Command;

    /// 执行 PowerShell CIM 查询，返回首行结果
    fn cim(class: &str, props: &[&str]) -> String {
        let select = props.join(",");
        let script = format!("(Get-CimInstance {class}).{select} -join ','");
        Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }

    let manufacturer = cim("Win32_ComputerSystemProduct", &["Vendor"]);
    let product_name = cim("Win32_ComputerSystemProduct", &["Name"]);
    let serial = cim("Win32_ComputerSystemProduct", &["SerialNumber"]);
    let bios_vendor = cim("Win32_BIOS", &["Manufacturer"]);
    let cpu = cim("Win32_Processor", &["Name"]);
    let memory_mb = cim("Win32_ComputerSystem", &["TotalPhysicalMemory"])
        .parse::<u64>()
        .map(|b| b / 1024 / 1024)
        .unwrap_or(0);
    let network = cim("Win32_NetworkAdapter", &["NetConnectionID"])
        .split(',')
        .find(|s| !s.trim().is_empty())
        .unwrap_or("e1000")
        .to_string();

    info!("读取 Windows 宿主 SMBIOS/WMI 成功");

    HardwareInfo {
        bios_vendor: nonempty(&bios_vendor, "HAAVK Group"),
        manufacturer: nonempty(&manufacturer, "HAAVK哈夫克集团"),
        product_name: nonempty(&product_name, "HAAVK全域算力终端"),
        serial: nonempty(&serial, "HAAVK-NODE-UNKNOWN"),
        cpu: nonempty(&cpu, "Mandel CPU"),
        memory_mb,
        network,
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
fn collect_impl() -> HardwareInfo {
    HardwareInfo::unknown()
}

fn nonempty(v: &str, fallback: &str) -> String {
    if v.trim().is_empty() { fallback.to_string() } else { v.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_has_brand() {
        let h = HardwareInfo::unknown();
        assert!(h.product_name.contains("HAAVK"));
    }
}
