use crate::image_data::{Error, Id, ImageData, Rgba, Size};
use crate::helper;
use crate::components::{Preview, Viewer, card, placeholder};

use iced::animation;
use iced::time::Instant;
use iced::widget::{center_x, container, row, scrollable, stack};
use iced::window;
use iced::{Animation, Element, Subscription, Task, Theme};
use iced::widget::scrollable::Viewport;
use iced::keyboard::Event;
use iced::keyboard::key::Key;
use iced::keyboard::key::Named;
use iced::event::{self, Event as IcedEvent};

use std::collections::HashMap;
use std::path::PathBuf;

pub struct Gallery {
    images: Vec<ImageData>,
    previews: HashMap<Id, Preview>,
    viewer: Viewer,
    now: Instant,
    image_dir: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenImageDirectory(PathBuf),
    ImagesListed(Result<Vec<ImageData>, Error>),
    ImagePoppedIn(Id),
    ImageDownloaded(Result<Rgba, Error>),
    ThumbnailDownloaded(Id, Result<Rgba, Error>),
    ThumbnailHovered(Id, bool),
    Open(Id),
    Close,
    Animate(Instant),
    ViewportChanged(Viewport),
    KeyPressed(Event),
}

impl Gallery {
    pub fn new() -> Self {
        Self {
            images: Vec::new(),
            previews: HashMap::new(),
            viewer: Viewer::new(),
            now: Instant::now(),
            image_dir: None,
        }
    }

    pub fn theme(&self) -> Theme {
        Theme::TokyoNight
    }

    pub fn subscription(&self) -> Subscription<Message> {
        let is_animating = self
            .previews
            .values()
            .any(|preview| preview.is_animating(self.now))
            || self.viewer.is_animating(self.now);

        let keyboard = event::listen().map(|event| {
            if let IcedEvent::Keyboard(keyboard_event) = event {
                Message::KeyPressed(keyboard_event)
            } else {
                Message::Animate(Instant::now())
            }
        });

        if is_animating {
            Subscription::batch([
                window::frames().map(Message::Animate),
                keyboard,
            ])
        } else {
            keyboard
        }
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::OpenImageDirectory(path) => {
                self.image_dir = Some(path.clone());
                let paths = helper::list_image_files(path.to_str().unwrap_or_default());
                Task::perform(
                    async move { ImageData::list_from_paths(paths).await },
                    Message::ImagesListed,
                )
            }
            Message::ImagesListed(Ok(images)) => {
                self.images = images;
                Task::none()
            }
            Message::ImagePoppedIn(id) => {
                let Some(image) = self
                    .images
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .cloned()
                else {
                    return Task::none();
                };

                let id = id.clone();
                Task::perform(
                    image.download(Size::Thumbnail {
                        width: Preview::WIDTH,
                        height: Preview::HEIGHT,
                    }),
                    move |result| Message::ThumbnailDownloaded(id, result),
                )
            }
            Message::ImageDownloaded(Ok(rgba)) => {
                self.viewer.show(rgba);
                Task::none()
            }
            Message::ThumbnailDownloaded(id, Ok(rgba)) => {
                let thumbnail = if let Some(preview) = self.previews.remove(&id) {
                    preview.load(rgba)
                } else {
                    Preview::ready(rgba)
                };

                let _ = self.previews.insert(id, thumbnail);
                Task::none()
            }
            Message::ThumbnailHovered(id, is_hovered) => {
                if let Some(preview) = self.previews.get_mut(&id) {
                    preview.toggle_zoom(is_hovered);
                }
                Task::none()
            }
            Message::Open(id) => {
                let Some(image) = self
                    .images
                    .iter()
                    .find(|candidate| candidate.id == id)
                    .cloned()
                else {
                    return Task::none();
                };

                let current_index = self.images.iter().position(|img| img.id == id);
                self.viewer.open();
                self.viewer.set_current_id(Some(id));
                self.viewer.set_current_index(current_index);
                Task::perform(
                    image.download(Size::Original),
                    Message::ImageDownloaded,
                )
            }
            Message::Close => {
                self.viewer.close();
                Task::none()
            }
            Message::Animate(now) => {
                self.now = now;
                Task::none()
            }
            Message::ViewportChanged(viewport) => {
                Task::none()
            }
            Message::KeyPressed(event) => {
                if let Event::KeyPressed { key, .. } = event {
                    if self.viewer.is_open(self.now) {
                        match key {
                            Key::Named(Named::ArrowLeft) => {
                                let current_index = self.viewer.current_index().unwrap();
                                if current_index > 0 {
                                    let prev_image = &self.images[current_index - 1];
                                    println!("Loading previous image: {:?}", prev_image);
                                    self.viewer.set_current_index(Some(current_index - 1));
                                    self.viewer.set_current_id(Some(prev_image.id));
                                    return Task::perform(
                                        prev_image.clone().download(Size::Original),
                                        Message::ImageDownloaded,
                                    );
                                }
                            }
                            Key::Named(Named::ArrowRight) => {
                                let current_index = self.viewer.current_index().unwrap();
                                if current_index < self.images.len() - 1 {
                                    let next_image = &self.images[current_index + 1];
                                    println!("Loading next image: {:?}", next_image);
                                    self.viewer.set_current_index(Some(current_index + 1));
                                    self.viewer.set_current_id(Some(next_image.id));
                                    return Task::perform(
                                        next_image.clone().download(Size::Original),
                                        Message::ImageDownloaded,
                                    );
                                }
                            }
                            Key::Named(Named::Escape) => {
                                self.viewer.close();
                            }
                            _ => {}
                        }
                    }
                }
                Task::none()
            }
            Message::ImagesListed(Err(error))
            | Message::ImageDownloaded(Err(error))
            | Message::ThumbnailDownloaded(_, Err(error)) => {
                dbg!(error);
                Task::none()
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let gallery = if self.images.is_empty() {
            row((0..=ImageData::LIMIT).map(|_| placeholder()))
        } else {
            row(self.images.iter().map(|image| {
                card(image, self.previews.get(&image.id), self.now)
            }))
        }
        .spacing(4)
        .wrap();

        let content = container(scrollable(center_x(gallery))
            .spacing(4)
            .on_scroll(Message::ViewportChanged))
            .padding(4);

        let viewer = self.viewer.view(self.now);

        stack![content, viewer].into()
    }
} 