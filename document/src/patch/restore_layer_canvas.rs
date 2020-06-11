use crate::color::*;
use crate::patch::IPatch;
use math::Extent2;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreLayerCanvasPatch<A>
where
	A: IColor, {
	pub target: Uuid,
	pub name: String,
	pub size: Extent2<u32>,
	pub albedo: Vec<A>,
	pub alpha: Vec<Alpha>,
	pub normal: Vec<Normal>,
}

impl<A> IPatch for RestoreLayerCanvasPatch<A>
where
	A: IColor, {}