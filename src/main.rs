use std::cmp::max;
use nu_ansi_term::{Color, Style};

use crate::os::mac_os::MacOS;
use crate::os::OS;
use crate::utils::{get_style_unset, SystemInfo};

mod os;
mod utils;

#[tokio::main]
async fn main() {
    #[cfg(target_os = "windows")]
    nu_ansi_term::enable_ansi_support()
        .expect("");

    let img = MacOS::get_image();
    let theme = MacOS::get_theme();

    let lines = img.get_lines();
    let max_len = lines
        .iter()
        .map(|(_, len)| len)
        .fold(0, |a, &b| max(a, b));

    let s = SystemInfo::new();
    let title = s.get_title().await.unwrap_or(String::new());
    let infos = s.get_infos().await;
    let infos_len = infos.len() + 2;

    let mut i = 0;
    for (line, len) in lines {
        let content = match i {
            0 => theme.render_title(&title).await,
            1 => "-".repeat(title.len()),
            i if (2..infos_len).contains(&i) =>
                infos
                    .get(i-2)
                    .map(|(k,v)|format!(
                        "{secondary}{0}{unset}: {1}",
                        k,
                        v,
                        secondary = theme.get_secondary().unwrap_or(&get_style_unset()).prefix(),
                        unset = get_style_unset().prefix(),
                    ))
                    .unwrap_or("".to_string()),
            i if i == infos_len + 1 => {
                format!(
                    "{}   {}   {}   {}   {}   {}   {}   {}   {}",
                    Style::new().on(Color::Black).prefix(),
                    Style::new().on(Color::Red).prefix(),
                    Style::new().on(Color::Green).prefix(),
                    Style::new().on(Color::Yellow).prefix(),
                    Style::new().on(Color::Blue).prefix(),
                    Style::new().on(Color::Purple).prefix(),
                    Style::new().on(Color::LightBlue).prefix(),
                    Style::new().on(Color::White).prefix(),
                    get_style_unset().prefix(),
                )
            },
            _ => "".to_string(),
        };

        println!("{}{}{content}", line, " ".repeat((max_len + 3 - len) as usize));

        i += 1;
    }
}
