mod immutable;

pub use immutable::ImmutableTexture;

use vulkano::image::ImageViewAccess;
use std::sync::Arc;

pub trait Texture {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync>;
}
