use iced::widget::{column, container, svg, text};
use iced::{Alignment, Color, ContentFit, Element, Length, Subscription, Task, Theme};

static SCAN_CARD_ICON: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64"
     fill="none" stroke="currentColor" stroke-width="3"
     stroke-linecap="round" stroke-linejoin="round">
  <!-- card body -->
  <rect x="13" y="19" width="38" height="26" rx="4"/>
  <!-- magnetic stripe -->
  <path d="M13 27h38"/>

  <!-- barcode -->
  <g fill="currentColor" stroke="none">
    <rect x="30.0" y="32.5" width="1.0" height="8"/>
    <rect x="31.6" y="32.5" width="0.7" height="8"/>
    <rect x="32.9" y="32.5" width="1.3" height="8"/>
    <rect x="34.8" y="32.5" width="0.7" height="8"/>
    <rect x="36.1" y="32.5" width="1.0" height="8"/>
    <rect x="37.7" y="32.5" width="0.7" height="8"/>
    <rect x="39.0" y="32.5" width="1.4" height="8"/>
    <rect x="41.0" y="32.5" width="0.7" height="8"/>
    <rect x="42.3" y="32.5" width="1.0" height="8"/>
    <rect x="43.9" y="32.5" width="0.7" height="8"/>
    <rect x="45.2" y="32.5" width="1.1" height="8"/>
  </g>

  <!-- corner brackets -->
  <path d="M4 12V8a4 4 0 0 1 4-4h4"/>
  <path d="M52 4h4a4 4 0 0 1 4 4v4"/>
  <path d="M60 52v4a4 4 0 0 1-4 4h-4"/>
  <path d="M12 60H8a4 4 0 0 1-4-4v-4"/>
</svg>
"##;

fn title(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.extended_palette().background.base.text),
    }
}
fn hint(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(Color {
            a: 0.65,
            ..theme.extended_palette().background.base.text
        }),
    }
}

pub struct LoginPage {}
impl LoginPage {
    pub fn view<'a, Message: 'a>() -> Element<'a, Message> {
        let icon = svg(svg::Handle::from_memory(SCAN_CARD_ICON))
            .width(Length::Fixed(320.0))
            .height(Length::Fixed(320.0))
            .content_fit(ContentFit::Contain)
            .style(|theme: &Theme, _status| svg::Style {
                color: Some(theme.extended_palette().primary.base.color),
            });

        let copy = column![
            text("Scan je schoolpas").size(56).style(title),
            text("Scan je barcode bij de scanner.").size(32).style(hint),
        ]
        .spacing(16)
        .align_x(Alignment::Center);

        let content = column![icon, copy].spacing(48).align_x(Alignment::Center);
        container(content).center(Length::Fill).into()
    }
}
