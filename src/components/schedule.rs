use chrono::{DateTime, Local};
use iced::widget::{self, Stack, container, pin, responsive};
use iced::{Element, Length, Pixels, Size, Theme};

use crate::components::lesson_card::Lesson;

const GAP: f32 = 2.0;
const MIN_HEIGHT: f32 = 3.0;
pub const START_TIME: Time = Time(8, 30);
pub const END_TIME: Time = Time(17, 00);

pub struct Time(pub u8, pub u8);

pub struct PlacedLesson {
    pub start_frac: f64,
    pub duration_frac: f64,
    pub lane: u16,
    pub lesson: Lesson,
}

pub struct Block {
    pub lanes: u16,
    pub lessons: Vec<PlacedLesson>,
}

pub struct Schedule {
    start_time: DateTime<Local>,
    end_time: DateTime<Local>,
    span: f64,
    blocks: Vec<Block>,
}

fn time_line_style(theme: &Theme) -> widget::rule::Style {
    let pallette = theme.extended_palette();
    widget::rule::Style {
        color: iced::Color {
            a: 0.7,
            ..pallette.secondary.strong.color
        },
        ..widget::rule::weak(theme)
    }
}

impl Schedule {
    pub fn new(lessons: &[Lesson], start_time: DateTime<Local>, end_time: DateTime<Local>) -> Self {
        let span = (end_time - start_time).num_milliseconds() as f64;

        Self {
            span,
            blocks: Self::build(lessons, start_time, end_time, span),
            start_time,
            end_time,
        }
    }

    fn self_frac(&self, t: DateTime<Local>) -> f64 {
        Self::frac(t, self.start_time, self.span)
    }
    fn frac(t: DateTime<Local>, start: DateTime<Local>, span: f64) -> f64 {
        (((t - start).num_milliseconds() as f64) / span).clamp(0.0, 1.0)
    }

    pub fn view<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        responsive(move |size| {
            let mut layers: Vec<Element<'a, Message>> = Vec::new();

            for block in &self.blocks {
                let lanes = block.lanes.max(1) as f32;
                let column = size.width.max(1.0) / lanes;

                for item in &block.lessons {
                    let top = size.height * item.start_frac as f32;
                    let bottom = size.height * (item.start_frac + item.duration_frac) as f32;

                    let card = container(item.lesson.view())
                        .width(Length::Fixed((column - GAP).max(1.0)))
                        .height(Length::Fixed((bottom - top - GAP).max(MIN_HEIGHT)));

                    layers.push(
                        pin(card)
                            .x(Pixels(column * item.lane as f32 + GAP / 2.0))
                            .y(Pixels(top + GAP / 2.0))
                            .into(),
                    );
                }
            }

            layers.push(self.current_time_line(size));
            Stack::with_children(layers).into()
        })
        .into()
    }

    fn current_time_line<'a, Message: 'a>(&self, size: Size) -> Element<'a, Message> {
        let now = Local::now();
        let frac = self.self_frac(now).clamp(0.0, 1.0);
        let y = size.height.max(1.0) * frac as f32;
        let width = size.width.max(1.0);

        pin(widget::rule::horizontal(GAP).style(time_line_style))
            .x(Pixels(0.0))
            .y(Pixels(y))
            .width(width)
            .into()
    }

    fn build(
        lessons: &[Lesson],
        start: DateTime<Local>,
        end: DateTime<Local>,
        span: f64,
    ) -> Vec<Block> {
        if span <= 0.0 {
            return Vec::new();
        }
        let frac = |t| Self::frac(t, start, span);

        let mut sorted: Vec<&Lesson> = lessons
            .iter()
            .filter(|lesson| {
                lesson.end_time >= lesson.start_time
                    && lesson.end_time > start
                    && lesson.start_time < end
            })
            .collect();
        sorted.sort_by_key(|lesson| (lesson.start_time, lesson.end_time));

        let mut blocks: Vec<Block> = Vec::new();
        let mut placed: Vec<PlacedLesson> = Vec::new();
        let mut lane_ends: Vec<DateTime<Local>> = Vec::new();
        let mut group_end: Option<DateTime<Local>> = None;

        for lesson in sorted {
            if group_end.is_some_and(|end| lesson.start_time >= end) {
                // create new group if there was a previous group and the new lesson starts after the old group end
                blocks.push(Block {
                    lanes: lane_ends.len() as u16,
                    lessons: std::mem::take(&mut placed),
                });
                lane_ends.clear();
                group_end = None;
            }

            let lane = match lane_ends.iter().position(|end| *end <= lesson.start_time) {
                // search for a lesson in the block which ends before the start of the current iteration lesson so that the current lesson can be appended in that lane
                Some(index) => index,
                None => {
                    // otherwise create a new lane
                    lane_ends.push(lesson.start_time);
                    lane_ends.len() - 1
                }
            };
            lane_ends[lane] = lesson.end_time; // update the lane end

            group_end = Some(group_end.map_or(lesson.end_time, |end| end.max(lesson.end_time))); // update the group end

            let start_frac = frac(lesson.start_time); // calculate the fracations and add the placed lesson into the list
            placed.push(PlacedLesson {
                start_frac,
                duration_frac: (frac(lesson.end_time) - start_frac).max(0.0),
                lane: lane as u16,
                lesson: lesson.clone(),
            });
        }

        if !placed.is_empty() {
            blocks.push(Block {
                lanes: lane_ends.len() as u16,
                lessons: placed,
            });
        }

        blocks
    }
}
