mod immutable;
mod target;

pub use immutable::ImmutableTexture;
pub use target::TargetTexture;

use std::sync::Arc;
use vulkano::image::ImageViewAccess;

pub trait Texture {
	fn image(&self) -> &Arc<dyn ImageViewAccess + Send + Sync>;
}
