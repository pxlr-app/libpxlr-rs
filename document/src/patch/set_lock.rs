use crate::patch::{IPatch, Patch, PatchMode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct SetLockPatch {
	pub target: Uuid,
	pub lock: bool,
}

impl IPatch for SetLockPatch {
	fn mode(&self) -> PatchMode {
		PatchMode::META
	}
}

pub trait ILockable {
	fn set_lock(&self, lock: bool) -> Result<(Patch, Patch), SetLockError>;
}

#[derive(Debug)]
pub enum SetLockError {
	Unchanged,
}

impl std::fmt::Display for SetLockError {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match *self {
			SetLockError::Unchanged => write!(f, "Could not set lock as value did not change."),
		}
	}
}

impl std::error::Error for SetLockError {
	fn cause(&self) -> Option<&dyn std::error::Error> {
		None
	}
}