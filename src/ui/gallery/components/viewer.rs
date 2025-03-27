use crate::core::{Id, Rgba};
use iced::animation;
use iced::time::Instant;
use iced::widget::{container, horizontal_space, image, mouse_area, opaque};
use iced::{Element, Fill, Theme, Animation};
use iced::color;

use crate::ui::gallery::Message;

pub struct Viewer {
    image: Option<image::Handle>,
    background_fade_in: Animation<bool>,
    image_fade_in: Animation<bool>,
    current_id: Option<Id>,
    current_index: Option<usize>,
}

impl Viewer {
    pub fn new() -> Self {
        Self {
            image: None,
            background_fade_in: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
            image_fade_in: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
            current_id: None,
            current_index: Some(0),
        }
    }

    pub fn current_id(&self) -> Option<Id> {
        self.current_id.clone()
    }

    pub fn set_current_id(&mut self, id: Option<Id>) {
        self.current_id = id;
    }

    pub fn current_index(&self) -> Option<usize> {
        self.current_index
    }

    pub fn set_current_index(&mut self, index: Option<usize>) {
        self.current_index = index;
    }

    pub fn is_open(&self, now: Instant) -> bool {
        self.background_fade_in.interpolate(0.0, 0.8, now) > 0.0
    }

    pub fn open(&mut self) {
        self.image = None;
        self.background_fade_in.go_mut(true);
    }

    pub fn show(&mut self, rgba: Rgba) {
        self.image = Some(image::Handle::from_rgba(
            rgba.width,
            rgba.height,
            rgba.pixels,
        ));
        self.background_fade_in.go_mut(true);
        self.image_fade_in.go_mut(true);
    }

    pub fn close(&mut self) {
        self.background_fade_in.go_mut(false);
        self.image_fade_in.go_mut(false);
        self.current_id = None;
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.background_fade_in.is_animating(now)
            || self.image_fade_in.is_animating(now)
    }

    pub fn view(&self, now: Instant) -> Element<'_, Message> {
        let opacity = self.background_fade_in.interpolate(0.0, 0.8, now);

        let image: Element<'_, _> = if let Some(handle) = &self.image {
            image(handle)
                .width(Fill)
                .height(Fill)
                .opacity(self.image_fade_in.interpolate(0.0, 1.0, now))
                .scale(self.image_fade_in.interpolate(1.5, 1.0, now))
                .into()
        } else {
            horizontal_space().into()
        };

        if opacity > 0.0 {
            opaque(
                mouse_area(
                    container(image)
                        .center(Fill)
                        .style(move |_theme| {
                            container::Style::default()
                                .background(color!(0x000000, opacity))
                        })
                        .padding(20),
                )
                .on_press(Message::Close),
            )
        } else {
            horizontal_space().into()
        }
    }
} 