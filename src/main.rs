mod core;
mod ui;

use ui::gallery::{Gallery, Message as GalleryMessage};
use iced::{Element, Theme, Task, Subscription};
use iced::widget::{button, container, text};
use std::env;
use std::path::PathBuf;

pub enum State {
    Landing { image_dir: String },
    Gallery(Gallery),
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadGallery,
    GalleryMessage(GalleryMessage),
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match (&mut *state, message) {
        (State::Landing { image_dir }, Message::LoadGallery) => {
            let gallery = Gallery::new();
            let path = PathBuf::from(image_dir.clone());
            *state = State::Gallery(gallery);
            Task::perform(
                async move { GalleryMessage::OpenImageDirectory(path) },
                Message::GalleryMessage,
            )
        }
        (State::Gallery(gallery), Message::GalleryMessage(gallery_msg)) => {
            gallery.update(gallery_msg).map(Message::GalleryMessage)
        }
        _ => Task::none(),
    }
}

fn view(state: &State) -> Element<Message> {
    match state {
        State::Landing { .. } => {
            container(
                button(text("Load Images"))
                    .on_press(Message::LoadGallery)
                    .padding(10)
            )
            .align_x(iced::Alignment::Center)
            .align_y(iced::Alignment::Center)
            .into()
        }
        State::Gallery(gallery) => {
            gallery.view().map(Message::GalleryMessage)
        }
    }
}

fn subscription(state: &State) -> Subscription<Message> {
    match state {
        State::Landing { .. } => Subscription::none(),
        State::Gallery(gallery) => gallery.subscription().map(Message::GalleryMessage),
    }
}

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_directory>", args[0]);
        std::process::exit(1);
    }

    let image_dir = args[1].clone();
    iced::application("Gallery - Iced", update, view)
        .subscription(subscription)
        .theme(|_| Theme::TokyoNight)
        .run_with(move || {
            let state = State::Landing { image_dir };
            (state, Task::none())
        })
} 