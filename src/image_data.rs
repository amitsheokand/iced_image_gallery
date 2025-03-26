use bytes::Bytes;
use tokio::task;

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Image {
    pub id: Id,
    path: PathBuf,
}

impl Image {
    pub const LIMIT: usize = 1000;

    pub async fn list() -> Result<Vec<Self>, Error> {
        let mut images = Vec::new();
        let mut id = 0;

        // Read images from the images directory
        if let Ok(entries) = std::fs::read_dir("images") {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().map_or(false, |ext| {
                        ["jpg", "jpeg", "png", "gif"].contains(&ext.to_str().unwrap_or("").to_lowercase().as_str())
                    }) {
                        images.push(Image {
                            id: Id(id),
                            path,
                        });
                        id += 1;
                    }
                }
            }
        }

        Ok(images)
    }

    pub async fn download(self, size: Size) -> Result<Rgba, Error> {
        let image = task::spawn_blocking(move || {
            Ok::<_, Error>(
                image::open(&self.path)?
                    .to_rgba8(),
            )
        })
        .await??;

        Ok(Rgba {
            width: image.width(),
            height: image.height(),
            pixels: Bytes::from(image.into_raw()),
        })
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub struct Id(u32);

#[derive(Clone)]
pub struct Rgba {
    pub width: u32,
    pub height: u32,
    pub pixels: Bytes,
}

impl fmt::Debug for Rgba {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rgba")
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Size {
    Original,
    Thumbnail { width: u32, height: u32 },
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Error {
    IOFailed(Arc<io::Error>),
    JoinFailed(Arc<task::JoinError>),
    ImageDecodingFailed(Arc<image::ImageError>),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IOFailed(Arc::new(error))
    }
}

impl From<task::JoinError> for Error {
    fn from(error: task::JoinError) -> Self {
        Self::JoinFailed(Arc::new(error))
    }
}

impl From<image::ImageError> for Error {
    fn from(error: image::ImageError) -> Self {
        Self::ImageDecodingFailed(Arc::new(error))
    }
}
