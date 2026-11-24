use std::{ops::AddAssign, time::Duration};

use chrono::{Local, Timelike};
use iced::{
    Element, Subscription, Task, Theme,
    application::IntoBoot,
    keyboard,
    wgpu::wgc::command::PassErrorScope::WriteTimestamp,
    widget::{self, column},
};
use tokio::time;

use crate::{
    api::{Api, ApiErrors, Lesson as ApiLesson},
    components::{END_TIME, Lesson, START_TIME, Schedule},
    config::ConfigManager,
    pages::{SchedulePage, SchedulePageMessage, StaticPages},
};

mod api;
mod components;
mod config;
mod pages;
mod typography;

type ApiSchedule = (String, Vec<Lesson>);

// #[derive(Debug, Copy, Clone)]
enum Messages {
    SchedulePage(SchedulePageMessage),
    KeyEvent(keyboard::Event),
    SwitchPage(Pages),
    ApiSetup(Result<Api, ApiErrors>),
    ApiLoaded(Result<Api, ApiErrors>),
    ApiScheduleReceived(Result<ApiSchedule, ApiErrors>),
    ApiRetry,
}
enum Pages {
    SchedulePage(SchedulePage),
    ErrorPage(String),
    LoadingPage,
    LoginPage,
    SetupPage,
    None,
}

struct App {
    page: Pages,
    input: String,
    config_manager: ConfigManager,
    is_set_up: bool,
    api: Option<Api>,
}

fn main() -> iced::Result {
    iced::application(App::new, App::update, App::view)
        .theme(Theme::Nightfly)
        .subscription(App::subscription)
        .run()
}

impl App {
    fn new() -> (Self, Task<Messages>) {
        let config_manager = ConfigManager::new();
        let config = config_manager.get();
        let is_set_up = !(config.app_url.is_empty() || config.app_token.is_empty());

        let page = match is_set_up {
            true => Pages::None,
            false => Pages::SetupPage,
        };

        let task = match is_set_up {
            true => Task::perform(
                Api::with_token(config.app_url.clone(), config.app_token.clone()),
                Messages::ApiLoaded,
            ),
            false => Task::none(),
        };

        (
            Self {
                page: page,
                input: "".into(),
                config_manager,
                is_set_up,
                api: None,
            },
            task,
        )
    }

    fn update(&mut self, message: Messages) -> Task<Messages> {
        match message {
            Messages::SwitchPage(page) => {
                self.page = page;
                Task::none()
            }

            // Schedulepage Logic
            Messages::SchedulePage(SchedulePageMessage::Finished)
                if matches!(self.page, Pages::SchedulePage(_)) =>
            {
                self.page = Pages::LoginPage;
                Task::none()
            }
            Messages::SchedulePage(msg)
                if let Pages::SchedulePage(schedule_page) = &mut self.page =>
            {
                schedule_page.update(msg).map(Messages::SchedulePage)
            }

            // Keyboard Logic
            Messages::KeyEvent(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(keyboard::key::Named::Enter),
                repeat: false,
                ..
            }) => {
                self.page = Pages::None;
                let task = match self.api.is_some() && self.is_set_up {
                    true => Task::perform(
                        request_lesson(self.api.clone().unwrap(), self.input.clone()),
                        Messages::ApiScheduleReceived,
                    ), //TODO: FIXME clone
                    false => Task::perform(set_up_api(self.input.clone()), Messages::ApiSetup),
                };
                self.input = "".into();

                task
            }
            Messages::KeyEvent(keyboard::Event::KeyPressed {
                modified_key: keyboard::Key::Character(c),
                repeat: false,
                ..
            }) => {
                self.input += &c;
                println!("{}", self.input);
                Task::none()
            }

            // Api Logic (positive)
            Messages::ApiSetup(Ok(api)) | Messages::ApiLoaded(Ok(api)) => {
                self.api = Some(api);
                self.page = Pages::LoginPage;
                self.is_set_up = true;

                let apiclone = self.api.clone().unwrap();
                self.config_manager.update(move |x| {
                    x.app_url = apiclone.hostname().into();
                    x.app_token = apiclone.token().into();
                });

                Task::none()
            }

            Messages::ApiScheduleReceived(Ok(schedule)) => {
                let config = self.config_manager.get();
                self.page = Pages::SchedulePage(SchedulePage::new(
                    &schedule.1,
                    schedule.0,
                    config.app_name.clone(),
                    config.schedule_timeout,
                ));
                Task::none()
            }

            // Api logic (negative)
            Messages::ApiRetry => {
                let config = self.config_manager.get();
                self.page = Pages::LoadingPage;

                Task::perform(
                    Api::with_token(config.app_url.clone(), config.app_token.clone()),
                    Messages::ApiLoaded,
                )
            }

            Messages::ApiSetup(Err(err)) => {
                eprintln!("{:?}", err);
                self.page = Pages::ErrorPage(
                    "Instellen mislukt, probeer opnieuw. Check de logs voor de error.".into(),
                );

                Task::perform(
                    time::sleep(self.config_manager.get().schedule_timeout),
                    |_| Messages::SwitchPage(Pages::SetupPage),
                )
            }

            Messages::ApiLoaded(Err(err)) => {
                eprintln!("{:?}", err);
                self.page = Pages::ErrorPage(
                    "Het lukte niet om met onze server te verbinden. We proberen het over een minuut opnieuw.".into(),
                );
                Task::perform(
                    time::sleep(self.config_manager.get().api_retry_duration),
                    |_| Messages::ApiRetry,
                )
            }

            Messages::ApiScheduleReceived(Err(err)) => {
                eprintln!("{:?}", err);
                self.page =
                    Pages::ErrorPage("Er ging iets mis met het ophalen van je rooster.".into());

                Task::perform(
                    time::sleep(self.config_manager.get().schedule_timeout),
                    |_| Messages::SwitchPage(Pages::LoginPage),
                )
            }

            _ => Task::none(),
        }
    }

    fn subscription(&self) -> Subscription<Messages> {
        let page_subscription = match &self.page {
            Pages::SchedulePage(page) => page.subscription().map(Messages::SchedulePage),
            _ => Subscription::none(),
        };
        let keyboard_subscription = keyboard::listen().map(Messages::KeyEvent);

        Subscription::batch([page_subscription, keyboard_subscription])
    }

    fn view(&self) -> Element<'_, Messages> {
        let config = self.config_manager.get();
        match &self.page {
            Pages::SchedulePage(page) => page.view(),
            Pages::LoginPage => StaticPages::view(),
            Pages::SetupPage => StaticPages::setup(&config.app_name),
            Pages::ErrorPage(message) => StaticPages::error(message),
            Pages::None => widget::space().into(),

            _ => todo!(),
        }
    }
}

async fn set_up_api(input: String) -> Result<Api, ApiErrors> {
    let split_input: Vec<&str> = input.split(",").collect();
    if split_input.len() != 2 {
        Err(ApiErrors::InputParsing)?;
    }
    println!("url{}\ntoken{}", split_input[0], split_input[1]);

    let hostname = split_input[0];
    let key = split_input[1];

    Api::with_key(hostname, key).await
}

async fn request_lesson(api: Api, id: String) -> Result<ApiSchedule, ApiErrors> {
    let timetable = api.get_timetable(&id).await?;
    let user = api.get_user(&id).await?;

    let lessons: Vec<Lesson> = timetable.lessons.iter().map(|x| x.into()).collect();

    Ok((user.first_name, lessons))
}
