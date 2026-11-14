use std::time::{Duration, Instant};

use chrono::{DateTime, Datelike, Local, TimeZone};
use iced::{Element, Length, Padding, Subscription, Task, widget, window};

use crate::components::{END_TIME, Lesson, START_TIME, Schedule, Sidepanel};

pub struct SchedulePage {
    schedule: Schedule,
    sidepanel: Sidepanel,

    begin: Instant,
    current: Instant,
    duration: Duration,
}

// #[derive(Debug, Copy, Clone)]
pub enum SchedulePageMessage {
    Tick(Instant),
    Finished,
}

const PADDING: u16 = 8;

type Type = Local;

impl SchedulePage {
    pub fn new(
        lessons: &[Lesson],
        user_name: String,
        app_name: String,
        duration: Duration,
    ) -> Self {
        let start_time = Self::at(START_TIME.0, START_TIME.1);
        let end_time = Self::at(END_TIME.0, END_TIME.1);

        let schedule = Schedule::new(lessons, start_time, end_time);
        let sidepanel = Sidepanel::new(app_name, user_name.clone(), None);

        let begin = Instant::now();

        SchedulePage {
            schedule,
            sidepanel,
            begin,
            current: begin,
            duration,
        }
    }

    pub fn update(&mut self, message: SchedulePageMessage) -> Task<SchedulePageMessage> {
        match message {
            SchedulePageMessage::Tick(tick) if self.is_finished(tick) => {
                Task::done(SchedulePageMessage::Finished)
            }
            SchedulePageMessage::Tick(tick) => {
                self.current = tick;
                Task::none()
            }
            _ => Task::none(),
        }
    }

    pub fn subscription(&self) -> Subscription<SchedulePageMessage> {
        window::frames().map(SchedulePageMessage::Tick)
    }

    pub fn view<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        let schedule: Element<'a, Message> = widget::container(self.schedule.view())
            .center(Length::FillPortion(10))
            .padding(Padding {
                left: ((PADDING - 2) / 2) as f32,
                ..(PADDING - 2).into()
            })
            .into();

        let sidepanel: Element<'a, Message> = widget::container(self.sidepanel.view())
            .center(Length::FillPortion(8))
            .padding(Padding {
                right: (PADDING / 2) as f32,
                ..(PADDING).into()
            })
            .into();

        let screen = widget::container(widget::row![sidepanel, schedule]).center(Length::Fill);
        let progress = self.current - self.begin;

        widget::column![
            screen,
            widget::container(widget::progress_bar(
                0.0..=(self.duration.as_millis() as f32),
                progress.as_millis() as f32,
            ))
            .height(Length::Fixed(PADDING as f32))
        ]
        .into()
    }

    pub fn at(h: u8, m: u8) -> DateTime<Type> {
        let now = Local::now();

        Local
            .with_ymd_and_hms(now.year(), now.month(), now.day(), h.into(), m.into(), 0)
            .single()
            .expect("local time should exist today")
    }
    fn is_finished(&self, tick: Instant) -> bool {
        return tick - self.begin > self.duration;
    }
}
