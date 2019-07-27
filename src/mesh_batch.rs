use crate::Context;
use std::sync::Arc;

pub struct MeshBatch {}
impl MeshBatch {
	pub fn new(_ctx: &Context) -> Arc<Self> {
		Arc::new(Self {})
	}
}
