mod image_data;
mod gallery;
mod helper;
mod components;

use gallery::{Gallery, Message};
use iced::Task;
use std::env;

fn main() -> iced::Result {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <image_directory>", args[0]);
        std::process::exit(1);
    }

    let image_dir = args[1].clone();
    iced::application("Gallery - Iced", Gallery::update, Gallery::view)
        .subscription(Gallery::subscription)
        .theme(Gallery::theme)
        .run_with(move || {
            let gallery = Gallery::new();
            (gallery, Task::perform(
                async move { Message::OpenImageDirectory(std::path::PathBuf::from(&image_dir)) },
                |msg| msg,
            ))
        })
} 