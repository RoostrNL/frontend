use std::borrow::Cow;

use iced::{
    Element, Font, Theme, font,
    widget::{Text, text},
};

pub const HEADING: u32 = 22;
pub const TITLE: u32 = 15;
pub const BODY: u32 = 13;
pub const META: u32 = 11;
pub const FINE: u32 = 9;

pub const DISPLAY: u32 = 36;
pub const BANNER: u32 = 56;
pub const BANNER_HINT: u32 = 32;

pub fn banner<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(BANNER)
}

pub fn banner_hint<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).style(muted).size(BANNER_HINT)
}

pub fn display<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(DISPLAY)
}

pub fn heading<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(HEADING)
}

pub fn title<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(TITLE).font(Font {
        weight: font::Weight::Medium,
        ..Font::default()
    })
}

pub fn strong<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(TITLE).font(Font {
        weight: font::Weight::Bold,
        ..Font::default()
    })
}

pub fn body<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(BODY)
}

pub fn meta<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).style(muted).size(META)
}

pub fn fine<'a>(content: impl iced::widget::text::IntoFragment<'a>) -> Text<'a> {
    text(content).size(FINE)
}
pub fn muted(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(theme.palette().text.scale_alpha(0.65)),
    }
}

