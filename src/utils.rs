use lazy_static::lazy_static;
use nu_ansi_term::Style;
use sysinfo::{System, Users};
use tokio::process::Command;

// pub fn get_title<'a>() -> &'a str {
// }

lazy_static!{
    static ref STYLE_UNSET: Style = Style::default().reset_before_style();
}

pub fn get_style_unset<'a>() -> &'a Style {
    &STYLE_UNSET
}

async fn exec(program: &str, args: Vec<&str>) -> String {
    let output = Command::new(program)
        .args(args)
        .output()
        .await
        .unwrap();
    String::from_utf8_lossy(&output.stdout).trim().to_string()
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

    pub async fn get_hostname(&self) -> String {
        #[cfg(target_os = "linux")]
        {
            exec("sh", vec![
                "-c",
                r#"sh -c "echo ${HOSTNAME:-$(hostname)}""#,
            ]).await
        }

        #[cfg(target_os = "windows")]
        {
            exec("whoami", vec![]).await.split(r"\").nth(0).unwrap_or("").to_string()
        }
    }

    pub async fn get_username(&self) -> String {
        #[cfg(target_os = "linux")]
        {
            exec("sh", vec![
                "-c",
                "id -un",
            ]).await
        }

        #[cfg(target_os = "windows")]
        {
            exec("whoami", vec![]).await.split(r"\").nth(1).unwrap_or("").to_string()
        }
    }

    pub async fn get_title(&self) -> String {
        #[cfg(target_os = "linux")]
        {
            format!(r"{1}@{0}", self.get_hostname().await, self.get_username().await)
        }

        #[cfg(target_os = "windows")]
        {
            format!(r"{0}\{1}", self.get_hostname().await, self.get_username().await)
        }
    }

    pub async fn get_infos(&self) -> Vec<(String, String)> {
        let mut infos = vec![];
        infos.push(("OS".to_string(), System::name().unwrap_or(String::new())));
        infos.push(("Kernel".to_string(), System::kernel_version().unwrap_or(String::new())));
        infos
    }
}
