use nu_ansi_term::{Color, Style};
use crate::os::{AsciiImage, Image, OS, Theme};

pub struct MacOS {
}

impl OS for MacOS {
    fn get_image() -> Image {
        Image::Ascii(
            AsciiImage {
                content: r#"${0}                    c.'
                 ,xNMM.
               .OMMMMo
               lMM"
     .;loddo:.  .olloddol;.
   cKMMMMMMMMMMNWMMMMMMMMMM0:
${1} .KMMMMMMMMMMMMMMMMMMMMMMMWd.
 XMMMMMMMMMMMMMMMMMMMMMMMX.
${2};MMMMMMMMMMMMMMMMMMMMMMMM:
:MMMMMMMMMMMMMMMMMMMMMMMM:
${3}.MMMMMMMMMMMMMMMMMMMMMMMMX.
 kMMMMMMMMMMMMMMMMMMMMMMMMWd.
 ${4}'XMMMMMMMMMMMMMMMMMMMMMMMMMMk
  'XMMMMMMMMMMMMMMMMMMMMMMMMK.
    ${5}kMMMMMMMMMMMMMMMMMMMMMMd
     ;KMMMMMMMWXXWMMMMMMMk.
       "cooc*"    "*coo'"
"#.to_string()
            }
        )
    }

    fn get_theme() -> Theme {
        Theme::new(vec![
            Style::new().fg(Color::Green).bold(),
            Style::new().fg(Color::Yellow).bold(),
            Style::new().fg(Color::Red).bold(),
            Style::new().fg(Color::Red).bold(),
            Style::new().fg(Color::Purple).bold(),
            Style::new().fg(Color::Blue).bold(),
        ])
    }
}