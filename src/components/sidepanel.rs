use iced::time::Instant;
use iced::widget::{column, container, progress_bar, row, space, text};
use iced::{Alignment, Border, Color, Element, Font, Length, Subscription, Task, Theme, window};

use crate::components::Lesson;
use crate::typography::muted;

pub const TIMEOUT: u16 = 10;

const CARD_PADDING: u16 = 12;
const SPACING: f32 = 12.0;

const TITLE_SIZE: f32 = 18.0;
const BODY_SIZE: f32 = 14.0;
const LABEL_SIZE: f32 = 12.0;
const TEMPERATURE_SIZE: f32 = 36.0;

const AVATAR_SIZE: f32 = 45.0;
const CARD_RADIUS: f32 = 12.0;
const BAR_THICKNESS: f32 = 4.0;

pub struct Sidepanel {
    app_name: String,
    user_name: String,
    next_lesson: Option<Lesson>,
}

impl Sidepanel {
    pub fn new(
        app_name: impl Into<String>,
        user_name: impl Into<String>,
        next_lesson: Option<Lesson>,
    ) -> Self {
        Self {
            app_name: app_name.into(),
            user_name: user_name.into(),
            next_lesson,
        }
    }

    pub fn view<'a, M: 'a>(&'a self) -> Element<'a, M> {
        let header = accent_card(
            row![
                avatar::<M>(&self.user_name),
                column![
                    text(self.app_name.as_str()).size(TITLE_SIZE),
                    text(format!("Welkom, {}", self.user_name))
                        .size(BODY_SIZE)
                        .style(muted),
                ]
                .spacing(2),
            ]
            .spacing(SPACING)
            .align_y(Alignment::Center),
        );

        let upcoming_lesson = match &self.next_lesson {
            Some(lesson) => Some(card(
                column![
                    text("Volgende les:").size(LABEL_SIZE).style(muted),
                    lesson.view(),
                ]
                .spacing(8),
            )),
            None => None,
        };

        let weather = card(
            column![
                text("Weer").size(LABEL_SIZE).style(muted),
                row![
                    text("--°").size(TEMPERATURE_SIZE),
                    text("Placeholder").size(BODY_SIZE),
                ]
                .spacing(SPACING)
                .align_y(Alignment::Center),
            ]
            .spacing(8),
        );

        column![header, upcoming_lesson, weather, space::vertical(),]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(SPACING)
            .into()
    }
}

fn avatar<'a, M: 'a>(user_name: &str) -> Element<'a, M> {
    let initial = user_name
        .chars()
        .next()
        .map(|first| first.to_uppercase().collect::<String>())
        .unwrap_or_else(|| "?".to_owned());

    container(
        text(initial)
            .size(TITLE_SIZE)
            .color(Color::WHITE)
            .font(Font::MONOSPACE),
    )
    .center(AVATAR_SIZE)
    .style(|theme: &Theme| container::Style {
        background: Some(theme.extended_palette().primary.base.color.into()),
        border: Border {
            radius: (AVATAR_SIZE / 2.0).into(),
            ..Border::default()
        },
        ..container::Style::default()
    })
    .into()
}

fn card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    styled_card(content, false)
}

fn accent_card<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    styled_card(content, true)
}

fn styled_card<'a, M: 'a>(content: impl Into<Element<'a, M>>, vibrant: bool) -> Element<'a, M> {
    container(content)
        .width(Length::Fill)
        .height(Length::Shrink)
        .padding(CARD_PADDING)
        .style(move |theme: &Theme| {
            let palette = theme.extended_palette();
            let background = if vibrant {
                &palette.background.weak
            } else {
                &palette.background.weakest
            };

            container::Style {
                background: Some(background.color.into()),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: CARD_RADIUS.into(),
                },
                ..container::Style::default()
            }
        })
        .into()
}
