use bytes::Bytes;
use tokio::task;

use std::fmt;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;

use crate::helper;

#[derive(Debug, Clone)]
pub struct Image {
    pub id: Id,
    path: PathBuf,
}

impl Image {
    pub const LIMIT: usize = 1000;

    pub async fn list() -> Result<Vec<Self>, Error> {
        Self::list_from_dir("images").await
    }

    pub async fn list_from_dir(dir: &str) -> Result<Vec<Self>, Error> {
        let paths = helper::list_image_files(dir);
        Self::list_from_paths(paths).await
    }

    pub async fn list_from_paths(paths: Vec<PathBuf>) -> Result<Vec<Self>, Error> {
        let mut images = Vec::new();
        
        for (id, path) in paths.into_iter().enumerate() {
            images.push(Image {
                id: Id(id as u32),
                path,
            });
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
