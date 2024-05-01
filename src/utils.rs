use std::path::Path;

use lazy_static::lazy_static;
use nu_ansi_term::Style;
use regex::Regex;
use sysinfo::{System, Users};
use tokio::fs::read_to_string;
use tokio::process::Command;

// pub fn get_title<'a>() -> &'a str {
// }

lazy_static! {
    static ref STYLE_UNSET: Style = Style::default().reset_before_style();
}

// region 扩展

pub trait PathExt {
    fn exists_and_is_dir(&self) -> bool;
}

impl PathExt for Path {
    fn exists_and_is_dir(&self) -> bool {
        self.exists() && self.is_dir()
    }
}

pub trait StrExt {
    fn exists_and_is_dir(&self) -> bool;
    fn exists(&self) -> bool;
}

impl StrExt for &str {
    fn exists_and_is_dir(&self) -> bool {
        Path::new(self).exists_and_is_dir()
    }
    fn exists(&self) -> bool {
        Path::new(self).exists()
    }
}

// endregion

pub fn get_style_unset<'a>() -> &'a Style {
    &STYLE_UNSET
}

async fn exec(program: &str, args: Vec<&str>) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .await
        .map(|output|String::from_utf8_lossy(&output.stdout).trim().to_string())
        .map(|s| Some(s))
        .unwrap_or(None)
}

async fn exec_sh(cmd: &str) -> Option<String> {
    #[cfg(not(target_os = "windows"))]
    return exec("sh", vec!["-c", cmd]).await;

    #[cfg(target_os = "windows")]
    return exec("cmd", vec!["/c", cmd]).await;
}

async fn read_file(path: &str) -> Option<String> {
    read_to_string(Path::new(path))
        .await
        .map(|s| Some(s))
        .unwrap_or(None)
}


pub struct SystemInfo {
    sysinfo: System,
    users: Users,
}

impl SystemInfo {
    pub fn new() -> Self {
        let sysinfo = System::new_all();
        let users = Users::new_with_refreshed_list();
        Self {
            sysinfo,
            users,
        }
    }

    pub async fn get_hostname(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            exec("sh", vec![
                "-c",
                r#"sh -c "echo ${HOSTNAME:-$(hostname)}""#,
            ]).await
        }

        #[cfg(target_os = "windows")]
        {
            exec("whoami", vec![])
                .await
                .and_then(|s|s.split(r"\").nth(0).map(|ss|ss.to_string()))
        }
    }

    pub async fn get_username(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            exec("sh", vec![
                "-c",
                "id -un",
            ]).await
        }

        #[cfg(target_os = "windows")]
        {
            exec("whoami", vec![])
                .await
                .and_then(|s|s.split(r"\").nth(1).map(|ss|ss.to_string()))
        }
    }

    pub async fn get_title(&self) -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let (Some(hostname), Some(username)) = (self.get_hostname().await, self.get_username().await) {
                Some(format!(r"{1}@{0}", hostname, username))
            } else {
                None
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let (Some(hostname), Some(username)) = (self.get_hostname().await, self.get_username().await) {
                Some(format!(r"{0}\{1}", hostname, username))
            } else {
                None
            }
        }
    }

    pub async fn get_os(&self) -> Option<String> {
        System::name()
    }

    pub async fn get_os_full(&self) -> Option<String> {
        Some(format!(
            "{}",
            System::long_os_version().unwrap_or("unknown".to_string()),
        ))
    }

    pub async fn get_model(&self) -> Option<String> {
        lazy_static! {
            static ref P_TBF: Regex = Regex::new(r"To Be Filled.*").unwrap();
            static ref P_OEM: Regex = Regex::new(r"OEM.*").unwrap();
            static ref P_OPEN_BSD: Regex = Regex::new(r"OpenBSD.*").unwrap();
        }
        let os = self.get_os().await.unwrap_or(String::new());
        let mut model = None;

        match os.as_str() {
            "Linux" => {
                if "/system/app".exists_and_is_dir() && "/system/priv-app".exists_and_is_dir() {
                    return exec_sh(r#"echo "$(getprop ro.product.brand) $(getprop ro.product.model)""#).await;
                } else if "/sys/devices/virtual/dmi/id/board_vendor".exists() || "/sys/devices/virtual/dmi/id/board_name".exists() {
                    let vendor = read_file("/sys/devices/virtual/dmi/id/board_vendor")
                        .await;
                    let name = read_file("/sys/devices/virtual/dmi/id/board_name")
                        .await;
                    model = if let (Some(v), Some(n)) = (vendor, name) {
                        Some(format!("{} {}", v, n))
                    } else {
                        None
                    };
                } else if "/sys/devices/virtual/dmi/id/product_name".exists() || "/sys/devices/virtual/dmi/id/product_version".exists() {
                    let name = read_file("/sys/devices/virtual/dmi/id/product_name")
                        .await;
                    let version = read_file("/sys/devices/virtual/dmi/id/product_version")
                        .await;
                    model = if let (Some(n), Some(v)) = (name, version) {
                        Some(format!("{} {}", n, v))
                    } else {
                        None
                    };
                } else if "/sys/firmware/devicetree/base/model".exists() {
                    model = read_file("/sys/firmware/devicetree/base/model")
                        .await;
                } else if "/tmp/sysinfo/model".exists() {
                    model = read_file("/tmp/sysinfo/model")
                        .await;
                }
            }
            "Mac OS X" | "macOS" => {
                if let Some(true) = exec_sh(r#"kextstat | grep -F -e "FakeSMC" -e "VirtualSMC""#).await.map(|s| s!="") {
                    model = Some(format!("Hackintosh (SMBIOS: {})", exec_sh("sysctl -n hw.model").await.unwrap_or(String::new())));
                } else {
                    model = exec_sh("sysctl -n hw.model").await;
                }
            }
            "iPhone OS" => {
                todo!()
            }
            "BSD" | "MINIX" => {
                model = exec_sh("sysctl -n hw.vendor hw.product").await;
            }
            "Windows" => {
                model = exec_sh("wmic computersystem get manufacturer,model")
                    .await
                    .and_then(|s|s.lines().nth(1).map(|ss|ss.to_string()))
                    .map(|s|s.split("\t").collect::<Vec<_>>().join(" "));
            }
            "Solaris" => {
                model = exec_sh("prtconf -b | awk -F':' '/banner-name/ {printf $2}'").await;
            }
            "AIX" => {
                model = exec_sh("/usr/bin/uname -M").await;
            }
            "FreeMiNT" => {
                lazy_static! {
                    static ref P_FREEMINT: Regex = Regex::new(r" (_MCH *)").unwrap();
                }
                model = exec_sh("sysctl -n hw.model")
                    .await
                    .map(|s|P_FREEMINT.replace(s.as_str(), "").trim().to_string());
            }
            _ => {}
        }

        model = model
            .map(|s| s.replace("To be filled by O.E.M.", ""))
            .map(|s| P_TBF.replace_all(&s, "").to_string())
            .map(|s| P_OEM.replace_all(&s, "").to_string())
            .map(|s| s.replace("Not Applicable", ""))
            .map(|s| s.replace("System Product Name", ""))
            .map(|s| s.replace("System Version", ""))
            .map(|s| s.replace("Undefined", ""))
            .map(|s| s.replace("Default string", ""))
            .map(|s| s.replace("Not Specified", ""))
            .map(|s| s.replace("Type1ProductConfigId", ""))
            .map(|s| s.replace("INVALID", ""))
            .map(|s| s.replace("All Series", ""))
            .map(|s| s.replace("�", ""))
            .map(|s| if s.contains("Standard PC") { format!("KVM/QEMU (${s})") } else { s })
            .map(|s| if P_OPEN_BSD.is_match(s.as_str()) { format!("vmm (${s})") } else { s });

        model
    }

    pub async fn get_cpu_names(&self) -> Option<Vec<String>> {
        let cpus = self.sysinfo.cpus();

        if cpus.len() == 0 {
            return None;
        }

        let mut result = vec![];

        for cpu in cpus {
            result.push(cpu.brand().to_string());
        }

        Some(result)
    }

    pub async fn get_gpu_names(&self) -> Option<Vec<String>> {
        #[cfg(not(target_os = "windows"))]
        {
            None
        }

        #[cfg(target_os = "windows")]
        {
            use winreg::enums::HKEY_LOCAL_MACHINE;
            use winreg::RegKey;
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            hklm
                .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\WinSAT")
                .and_then(|key| key.get_value("PrimaryAdapterString"))
                .map(|name| Some(vec![name]))
                .unwrap_or(None)
        }
    }

    pub async fn get_infos(&self) -> Vec<(String, String)> {
        let mut infos = vec![];

        // OS
        if let Some(os) = self.get_os_full().await {
            infos.push(("OS".to_string(), os));
        }

        // Model
        if let Some(model) = self.get_model().await {
            infos.push(("Model".to_string(), model));
        }

        // Host
        if let Some(host) = System::host_name() {
            infos.push(("Host".to_string(), host));
        }

        // Kernel
        if let Some(kernel_version) = System::kernel_version() {
            infos.push(("Kernel".to_string(), kernel_version));
        }

        // Uptime
        if let uptime = System::uptime() {
            infos.push(("Uptime".to_string(), uptime.to_string()));
        }

        // Kernel
        if let Some(kernel_version) = System::kernel_version() {
            infos.push(("Kernel".to_string(), kernel_version));
        }

        // Arch
        if let Some(cpu_arch) = System::cpu_arch() {
            infos.push(("Arch".to_string(), cpu_arch));
        }

        infos.push(("Shell".to_string(), System::distribution_id()));

        // CPU
        if let Some(names) = self.get_cpu_names().await {
            if names.len() == 1 {
                infos.push(("CPU".to_string(), names[0].clone()));
            } else if names.iter().all(|i| i.eq(&names[0])) {
                infos.push(("CPU".to_string(), format!("{} * {}", names[0], names.len())));
            } else {
                for i in 0..names.len() {
                    infos.push((format!("CPU-{}/{}", i, names.len()), names[i].to_string()));
                }
            }
        }

        // GPU
        if let Some(names) = self.get_gpu_names().await {
            if names.len() == 1 {
                infos.push(("GPU".to_string(), names[0].clone()));
            } else {
                for i in 0..names.len() {
                    infos.push((format!("GPU-{}", i), names[i].to_string()));
                }
            }
        }

        infos
    }
}
