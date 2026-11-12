use std::{ops::AddAssign, time::Duration};

use iced::{
    Element, Subscription, Task, Theme,
    application::IntoBoot,
    keyboard,
    widget::{self, column},
};

use crate::{
    components::Lesson,
    pages::{LoginPage, SchedulePage, SchedulePageMessage},
};

mod components;
mod pages;
mod typography;

const USER_NAME: &str = "Jurn";

// #[derive(Debug, Copy, Clone)]
enum Message {
    SchedulePage(SchedulePageMessage),
    KeyEvent(keyboard::Event),
    InputFinished,
}
enum Page {
    SchedulePage(SchedulePage),
    LoginPage,
}

struct App {
    page: Page,
    input: String,
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(Theme::Nightfly)
        .subscription(App::subscription)
        .run()
}

impl App {
    fn new() -> Self {
        Self {
            page: Page::SchedulePage(
                SchedulePage::new(&demo_lessons(), USER_NAME.into(), Duration::from_secs(10))
                    .into(),
            ),
            input: "".into(),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SchedulePage(SchedulePageMessage::Finished)
                if matches!(self.page, Page::SchedulePage(_)) =>
            {
                self.page = Page::LoginPage;
                Task::none()
            }
            Message::SchedulePage(msg)
                if let Page::SchedulePage(schedule_page) = &mut self.page =>
            {
                schedule_page.update(msg).map(Message::SchedulePage)
            }

            Message::KeyEvent(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Enter),
                repeat: false,
                ..
            }) => {
                println!("ENTER");
                self.input = "".into();
                Task::done(Message::InputFinished)
            }
            Message::KeyEvent(keyboard::Event::KeyPressed {
                modified_key: keyboard::Key::Character(c),
                repeat: false,
                ..
            }) => {
                self.input += &c;
                println!("{}", self.input);
                Task::none()
            }

            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let page_subscription = match &self.page {
            Page::SchedulePage(page) => page.subscription().map(Message::SchedulePage),
            _ => Subscription::none(),
        };
        let keyboard_subscription = keyboard::listen().map(Message::KeyEvent);

        Subscription::batch([page_subscription, keyboard_subscription])
    }

    fn view(&self) -> Element<'_, Message> {
        match &self.page {
            Page::SchedulePage(page) => page.view(),
            Page::LoginPage => LoginPage::view(),
        }
    }
}

fn demo_lessons() -> Vec<Lesson> {
    let slot = |index: u16,
                (start_h, start_m): (u8, u8),
                (end_h, end_m): (u8, u8),
                title: &str,
                teacher: &str,
                location: &str| Lesson {
        title: title.into(),
        teacher: teacher.into(),
        location: location.into(),
        lesson_index: Some(index),
        start_time: SchedulePage::at(start_h, start_m),
        end_time: SchedulePage::at(end_h, end_m),
    };

    vec![
        slot(1, (8, 30), (9, 20), "nat", "vss", "019"),
        slot(1, (8, 30), (10, 10), "nat", "vss", "019"),
        slot(2, (9, 20), (10, 0), "schk", "sho", "109"),
        slot(3, (10, 30), (11, 20), "schk", "sho", "109"),
        slot(3, (10, 30), (12, 10), "schk", "sho", "109"),
        slot(4, (11, 20), (12, 0), "nat", "vss", "019"),
        slot(5, (13, 30), (14, 20), "nat", "vss", "019"),
    ]
}
