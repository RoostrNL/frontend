use std::borrow::Cow;

use iced::widget::{column, container, svg};
use iced::{Alignment, ContentFit, Element, Length, Subscription, Task, Theme};

use crate::typography;

static SCAN_CARD: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><path fill="currentColor" d="M3 7q-.425 0-.712-.288T2 6V3q0-.425.288-.712T3 2h3q.425 0 .713.288T7 3t-.288.713T6 4H4v2q0 .425-.288.713T3 7m0 15q-.425 0-.712-.288T2 21v-3q0-.425.288-.712T3 17t.713.288T4 18v2h2q.425 0 .713.288T7 21t-.288.713T6 22zm15 0q-.425 0-.712-.288T17 21t.288-.712T18 20h2v-2q0-.425.288-.712T21 17t.713.288T22 18v3q0 .425-.288.713T21 22zm2.288-15.288Q20 6.425 20 6V4h-2q-.425 0-.712-.288T17 3t.288-.712T18 2h3q.425 0 .713.288T22 3v3q0 .425-.288.713T21 7t-.712-.288M17.5 19v-1.5H19V19zm0-3v-1.5H19V16zM16 17.5V16h1.5v1.5zM14.5 19v-1.5H16V19zM13 17.5V16h1.5v1.5zm3-3V13h1.5v1.5zM14.5 16v-1.5H16V16zM13 14.5V13h1.5v1.5zm1-3.5q-.425 0-.712-.288T13 10V6q0-.425.288-.712T14 5h4q.425 0 .713.288T19 6v4q0 .425-.288.713T18 11zm-8 8q-.425 0-.712-.288T5 18v-4q0-.425.288-.712T6 13h4q.425 0 .713.288T11 14v4q0 .425-.288.713T10 19zm0-8q-.425 0-.712-.288T5 10V6q0-.425.288-.712T6 5h4q.425 0 .713.288T11 6v4q0 .425-.288.713T10 11zm.5 6.5h3v-3h-3zm0-8h3v-3h-3zm8 0h3v-3h-3z"/></svg>"##;

static SCAN_QR: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><!-- Icon from Material Symbols by Google - https://github.com/google/material-design-icons/blob/master/LICENSE --><path fill="currentColor" d="M3 7q-.425 0-.712-.288T2 6V3q0-.425.288-.712T3 2h3q.425 0 .713.288T7 3t-.288.713T6 4H4v2q0 .425-.288.713T3 7m0 15q-.425 0-.712-.288T2 21v-3q0-.425.288-.712T3 17t.713.288T4 18v2h2q.425 0 .713.288T7 21t-.288.713T6 22zm15 0q-.425 0-.712-.288T17 21t.288-.712T18 20h2v-2q0-.425.288-.712T21 17t.713.288T22 18v3q0 .425-.288.713T21 22zm2.288-15.288Q20 6.425 20 6V4h-2q-.425 0-.712-.288T17 3t.288-.712T18 2h3q.425 0 .713.288T22 3v3q0 .425-.288.713T21 7t-.712-.288M17.5 19v-1.5H19V19zm0-3v-1.5H19V16zM16 17.5V16h1.5v1.5zM14.5 19v-1.5H16V19zM13 17.5V16h1.5v1.5zm3-3V13h1.5v1.5zM14.5 16v-1.5H16V16zM13 14.5V13h1.5v1.5zm1-3.5q-.425 0-.712-.288T13 10V6q0-.425.288-.712T14 5h4q.425 0 .713.288T19 6v4q0 .425-.288.713T18 11zm-8 8q-.425 0-.712-.288T5 18v-4q0-.425.288-.712T6 13h4q.425 0 .713.288T11 14v4q0 .425-.288.713T10 19zm0-8q-.425 0-.712-.288T5 10V6q0-.425.288-.712T6 5h4q.425 0 .713.288T11 6v4q0 .425-.288.713T10 11zm.5 6.5h3v-3h-3zm0-8h3v-3h-3zm8 0h3v-3h-3z"/></svg>
"##;

static ERROR: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24"><!-- Icon from Material Symbols by Google - https://github.com/google/material-design-icons/blob/master/LICENSE --><path fill="currentColor" d="M12 17q.425 0 .713-.288T13 16q0-.425-.288-.713T12 15q-.425 0-.713.288T11 16q0 .425.288.713T12 17Zm0 5q-2.075 0-3.9-.788t-3.175-2.137q-1.35-1.35-2.137-3.175T2 12q0-2.075.788-3.9t2.137-3.175q1.35-1.35 3.175-2.137T12 2q2.075 0 3.9.788t3.175 2.137q1.35 1.35 2.138 3.175T22 12q0 2.075-.788 3.9t-2.137 3.175q-1.35 1.35-3.175 2.138T12 22Zm0-9q.425 0 .713-.288T13 12V8q0-.425-.288-.713T12 7q-.425 0-.713.288T11 8v4q0 .425.288.713T12 13Z"/></svg>"##;

pub struct StaticPages {}

fn build_page<'a, Message: 'a>(
    svg_data: &'static [u8],
    title_text: impl Into<Cow<'a, str>>,
    hint_text: impl Into<Cow<'a, str>>,
) -> Element<'a, Message> {
    let title_text: Cow<'a, str> = title_text.into();
    let hint_text: Cow<'a, str> = hint_text.into();
    let icon = svg(svg::Handle::from_memory(svg_data))
        .width(Length::Fixed(320.0))
        .height(Length::Fixed(320.0))
        .content_fit(ContentFit::Contain)
        .style(|theme: &Theme, _status| svg::Style {
            color: Some(theme.extended_palette().primary.base.color),
        });

    let copy = column![
        typography::banner(title_text),
        typography::banner_hint(hint_text),
    ]
    .spacing(16)
    .align_x(Alignment::Center);

    let content = column![icon, copy].spacing(48).align_x(Alignment::Center);
    container(content).center(Length::Fill).into()
}
impl StaticPages {
    pub fn view<'a, Message: 'a>() -> Element<'a, Message> {
        build_page(
            SCAN_CARD,
            "Scan je schoolpas",
            "Scan je barcode bij de scanner.",
        )
    }

    pub fn setup<'a, Message: 'a>(app_name: &'a str) -> Element<'a, Message> {
        let hint_text =
            format!("Scan de QR-Code uit de {app_name} interface om dit apparaat in te stellen.");
        build_page(SCAN_QR, "Scan de setup code.", hint_text)
    }

    pub fn error<'a, Message: 'a>(error: &'a str) -> Element<'a, Message> {
        build_page(ERROR, "Er is iets mis gegaan :(", error)
    }
}
