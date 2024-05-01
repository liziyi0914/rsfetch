use lazy_static::lazy_static;
use nu_ansi_term::Style;
use regex::Regex;

use crate::os::mac_os::MacOS;
use crate::utils::get_style_unset;

lazy_static!{
    static ref ASCII: Regex = Regex::new(r"\$\{(?<index>\d+)\}").unwrap();
    static ref STYLE_UNSET: Style = Style::default().reset_before_style();
}

pub mod windows;
pub mod mac_os;

pub struct Theme {
    styles: Vec<Style>,
}

impl Theme {
    pub fn new(styles: Vec<Style>) -> Self {
        Self {
            styles
        }
    }
    pub fn get_primary(&self) -> Option<&Style> {
        self.get_style(0)
    }

    pub fn get_secondary(&self) -> Option<&Style> {
        self.get_style(1)
    }

    pub fn get_style(&self, index: usize) -> Option<&Style> {
        self.styles.get(index)
    }

    pub async fn render_title(&self, title: &String) -> String {
        #[cfg(target_os = "linux")]
        {
            let parts: Vec<_> = title.split(r"@").collect();
            format!(
                r"{style_primary}{0}{style_unset}@{style_primary}{1}{style_unset}",
                parts[0],
                parts[1],
                style_unset = get_style_unset().prefix(),
                style_primary = self.primary_style.prefix()
            )
        }

        #[cfg(target_os = "windows")]
        {
            let parts: Vec<_> = title.split(r"\").collect();
            format!(
                r"{style_primary}{0}{style_unset}\{style_primary}{1}{style_unset}",
                parts[0],
                parts[1],
                style_unset = get_style_unset().prefix(),
                style_primary = self.get_primary().unwrap_or(&STYLE_UNSET).prefix()
            )
        }
    }
}

pub trait OS {
    fn get_image() -> Image;
    fn get_theme() -> Theme;
}

pub enum Image {
    Ascii(AsciiImage),
}

impl Image {
    pub fn get_lines(&self) -> Vec<(String, u64)> {
        match self {
            Image::Ascii(ascii) => {
                let s = ascii.content.clone();
                let mut style = Style::default();
                let mut max_len = 0;
                let mut contents = vec![];
                for l in s.lines() {
                    let mut line = l.to_string();

                    let len = ASCII.split(&line)
                        .map(|i| i.len() as u64)
                        .fold(0, |acc, i| acc + i);

                    if len > max_len {
                        max_len = len;
                    }

                    while let Some(cap) = ASCII.captures(&line) {
                        let i: usize = cap.name("index").map(|s| s.as_str()).unwrap_or("0").parse().unwrap();
                        style = MacOS::get_theme().get_style(i).unwrap_or(&STYLE_UNSET).clone();
                        line = line.replacen(format!("${{{}}}", i).as_str(), format!("{}{}", style.suffix().to_string(), style.prefix().to_string()).as_str(), 1);
                    }

                    contents.push((format!("{}{}{}", style.prefix(), line, STYLE_UNSET.prefix()), len));
                }

                contents
            }
        }
    }
}

pub struct AsciiImage {
    pub content: String,
}