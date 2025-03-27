use crate::core::{Id, ImageData, Rgba};
use iced::animation;
use iced::time::Instant;
use iced::widget::{button, container, horizontal_space, image, mouse_area, pop};
use iced::{ContentFit, Element, Fill, Theme, Animation};

use crate::ui::gallery::Message;

pub fn card<'a>(
    metadata: &'a ImageData,
    preview: Option<&'a Preview>,
    now: Instant,
) -> Element<'a, Message> {
    let image = if let Some(preview) = preview {
        let thumbnail: Element<'_, _> =
            if let Preview::Ready { thumbnail, .. } = &preview {
                image(&thumbnail.handle)
                    .width(Fill)
                    .height(Fill)
                    .content_fit(ContentFit::Contain)
                    .opacity(thumbnail.fade_in.interpolate(0.0, 1.0, now))
                    .scale(thumbnail.zoom.interpolate(1.0, 1.02, now))
                    .into()
            } else {
                horizontal_space().into()
            };

        thumbnail
    } else {
        horizontal_space().into()
    };

    let card = mouse_area(
        container(image)
            .width(Preview::WIDTH)
            .height(Preview::HEIGHT)
            .style(container::dark),
    )
    .on_enter(Message::ThumbnailHovered(metadata.id, true))
    .on_exit(Message::ThumbnailHovered(metadata.id, false));

    if let Some(preview) = preview {
        let is_thumbnail = matches!(preview, Preview::Ready { .. });

        button(card)
            .on_press_maybe(is_thumbnail.then_some(Message::Open(metadata.id)))
            .padding(0)
            .style(button::text)
            .into()
    } else {
        pop(card)
            .on_show(|_| Message::ImagePoppedIn(metadata.id))
            .into()
    }
}

pub fn placeholder<'a>() -> Element<'a, Message> {
    container(horizontal_space())
        .width(Preview::WIDTH)
        .height(Preview::HEIGHT)
        .style(container::dark)
        .into()
}

#[derive(Debug, Clone)]
pub enum Preview {
    Loading,
    Ready {
        thumbnail: Thumbnail,
    },
}

#[derive(Debug, Clone)]
pub struct Thumbnail {
    pub handle: image::Handle,
    pub fade_in: Animation<bool>,
    pub zoom: Animation<bool>,
}

impl Preview {
    pub const WIDTH: u32 = 360;
    pub const HEIGHT: u32 = 360;

    pub fn ready(rgba: Rgba) -> Self {
        Self::Ready {
            thumbnail: Thumbnail::new(rgba),
        }
    }

    pub fn load(self, rgba: Rgba) -> Self {
        Self::Ready {
            thumbnail: Thumbnail::new(rgba),
        }
    }

    pub fn toggle_zoom(&mut self, enabled: bool) {
        if let Self::Ready { thumbnail, .. } = self {
            thumbnail.zoom.go_mut(enabled);
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        match &self {
            Self::Ready { thumbnail, .. } => {
                thumbnail.fade_in.is_animating(now)
                    || thumbnail.zoom.is_animating(now)
            }
            Self::Loading => false,
        }
    }
}

impl Thumbnail {
    pub fn new(rgba: Rgba) -> Self {
        Self {
            handle: image::Handle::from_rgba(
                rgba.width,
                rgba.height,
                rgba.pixels,
            ),
            fade_in: Animation::new(false).quick().go(true),
            zoom: Animation::new(false)
                .quick()
                .easing(animation::Easing::EaseInOut),
        }
    }
} 