use chrono::format::Colons::Colon;
use chrono::{DateTime, Local, Timelike};
use iced::widget;
use iced::{Alignment, Border, Element, Font, Length, Theme};

use crate::api::Lesson as ApiLesson;
use crate::pages::SchedulePage;
use crate::typography;

pub const TIME_FORMAT: &str = "%I:%M";

const SPACING: f32 = 2.0;
const PADDING: f32 = 8.0;
const RADIUS: f32 = 10.0;
const BORDER: f32 = 1.0;
const ACCENT: f32 = 6.0;

#[derive(Clone, Debug)]
pub struct Lesson {
    pub title: String,
    pub teacher: String,
    pub location: String,
    pub lesson_index: Option<u16>,
    pub start_time: DateTime<Local>,
    pub end_time: DateTime<Local>,
}

impl From<&ApiLesson> for Lesson {
    fn from(l: &ApiLesson) -> Self {
        let start_time = l.start.with_timezone(&Local);
        let end_time = l.end.with_timezone(&Local);

        let start_time = SchedulePage::at(start_time.hour() as u8, start_time.minute() as u8);
        let end_time = SchedulePage::at(end_time.hour() as u8, end_time.minute() as u8);

        Lesson {
            title: l.subjects.join(", "),
            teacher: l.teachers.join(", "),
            location: l.locations.join(", "),
            lesson_index: (l.start_time_slot > 0).then_some(l.start_time_slot as u16),
            start_time: start_time,
            end_time: end_time,
        }
    }
}
impl From<ApiLesson> for Lesson {
    fn from(value: ApiLesson) -> Self {
        Lesson::from(&value)
    }
}

fn container_style(theme: &Theme) -> widget::container::Style {
    let palette = theme.extended_palette();

    widget::container::Style {
        background: Some(palette.background.weak.color.into()),
        text_color: Some(palette.background.weak.text),
        border: Border {
            color: palette.background.strong.color,
            width: BORDER,
            radius: RADIUS.into(),
        },
        ..widget::container::Style::default()
    }
}

fn rule_style(theme: &Theme) -> widget::rule::Style {
    let pallette = theme.extended_palette();
    widget::rule::Style {
        fill_mode: widget::rule::FillMode::Full,
        radius: (ACCENT / 2.0).into(),
        color: pallette.primary.weak.color,
        ..widget::rule::default(theme)
    }
}

impl Default for Lesson {
    fn default() -> Self {
        let now = Local::now();

        Self {
            title: "Nederlands".into(),
            teacher: "Braams".into(),
            location: "CLD".into(),
            lesson_index: Some(3u16),
            start_time: now,
            end_time: now,
        }
    }
}

impl Lesson {
    fn time_text<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        let start_text = self.start_time.format(TIME_FORMAT).to_string();
        let end_text = self.end_time.format(TIME_FORMAT).to_string();

        widget::column![typography::body(start_text), typography::meta(end_text),]
            .align_x(Alignment::Center)
            .into()
    }

    fn separator<'a, Message: 'a>() -> Element<'a, Message> {
        widget::rule::vertical(ACCENT).style(rule_style).into()
    }

    fn body<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        let title_text = typography::title(self.title.as_str());
        let teacher_text = typography::meta(self.teacher.as_str());

        widget::column![title_text, teacher_text]
            .width(Length::Fill)
            // .spacing(SPACING)
            .into()
    }

    fn right<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        let mut right: widget::Column<'a, Message> = widget::column![]
            // .spacing(SPACING)
            .align_x(Alignment::End);

        if !self.location.is_empty() {
            right = right.push(typography::body(self.location.as_str()));
        }

        if let Some(index) = self.lesson_index {
            right = right.push(typography::meta(format!("{index}e")));
        }

        right.into()
    }

    pub fn view<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        let content = widget::row![
            self.time_text(),
            Self::separator(),
            self.body(),
            self.right(),
        ]
        .spacing(PADDING)
        .align_y(Alignment::Start);

        widget::container(content)
            .padding(PADDING)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_y(Alignment::Start)
            .clip(true)
            .style(container_style)
            .into()
    }
}
