use crate::document::IDocument;
use crate::parser;
use crate::patch::*;
use crate::INode;
use math::{Extent2, Vec2};
use nom::IResult;
use serde::{Deserialize, Serialize};
use std::io;
use std::rc::Rc;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Note {
	pub id: Uuid,
	pub is_visible: bool,
	pub is_locked: bool,
	pub note: Rc<String>,
	pub position: Rc<Vec2<f32>>,
}

impl Note {
	pub fn new(id: Option<Uuid>, note: &str, position: Vec2<f32>) -> Note {
		Note {
			id: id.or(Some(Uuid::new_v4())).unwrap(),
			is_visible: true,
			is_locked: false,
			note: Rc::new(note.to_owned()),
			position: Rc::new(position),
		}
	}
}

impl IDocument for Note {
	fn position(&self) -> Vec2<f32> {
		*(self.position).clone()
	}
}

impl INode for Note {
	fn is_visible(&self) -> bool {
		self.is_visible
	}
	fn is_locked(&self) -> bool {
		self.is_locked
	}
}

impl<'a> Renamable<'a> for Note {
	fn rename(&self, new_name: &'a str) -> Result<(Patch, Patch), RenameError> {
		if *self.note == new_name {
			Err(RenameError::Unchanged)
		} else {
			Ok((
				Patch::Rename(RenamePatch {
					target: self.id,
					name: new_name.to_owned(),
				}),
				Patch::Rename(RenamePatch {
					target: self.id,
					name: (*self.note).to_owned(),
				}),
			))
		}
	}
}

impl IVisible for Note {
	fn set_visibility(&self, visible: bool) -> Result<(Patch, Patch), SetVisibilityError> {
		if self.is_visible == visible {
			Err(SetVisibilityError::Unchanged)
		} else {
			Ok((
				Patch::SetVisibility(SetVisibilityPatch {
					target: self.id,
					visibility: visible,
				}),
				Patch::SetVisibility(SetVisibilityPatch {
					target: self.id,
					visibility: self.is_visible,
				}),
			))
		}
	}
}

impl ILockable for Note {
	fn set_lock(&self, lock: bool) -> Result<(Patch, Patch), SetLockError> {
		if self.is_locked == lock {
			Err(SetLockError::Unchanged)
		} else {
			Ok((
				Patch::SetLock(SetLockPatch {
					target: self.id,
					lock: lock,
				}),
				Patch::SetLock(SetLockPatch {
					target: self.id,
					lock: self.is_locked,
				}),
			))
		}
	}
}

impl IPatchable for Note {
	fn patch(&self, patch: &Patch) -> Option<Box<Self>> {
		if patch.target() == self.id {
			return match patch {
				Patch::Rename(patch) => Some(Box::new(Note {
					id: self.id,
					is_visible: self.is_visible,
					is_locked: self.is_locked,
					note: Rc::new(patch.name.clone()),
					position: self.position.clone(),
				})),
				Patch::SetVisibility(patch) => Some(Box::new(Note {
					id: self.id,
					is_visible: patch.visibility,
					is_locked: self.is_locked,
					note: self.note.clone(),
					position: self.position.clone(),
				})),
				Patch::SetLock(patch) => Some(Box::new(Note {
					id: self.id,
					is_visible: self.is_visible,
					is_locked: patch.lock,
					note: self.note.clone(),
					position: self.position.clone(),
				})),
				_ => None,
			};
		}
		return None;
	}
}

impl parser::v0::PartitionTableParse for Note {
	type Output = Note;

	fn parse<'b, S>(
		_index: &parser::v0::PartitionIndex,
		row: &parser::v0::PartitionTableRow,
		_storage: &mut S,
		bytes: &'b [u8],
	) -> IResult<&'b [u8], Self::Output>
	where
		S: io::Read + io::Seek,
	{
		Ok((
			bytes,
			Note {
				id: row.id,
				is_visible: row.is_visible,
				is_locked: row.is_locked,
				note: Rc::new(String::from(&row.name)),
				position: Rc::new(row.position),
			},
		))
	}

	fn write<S>(&self, index: &mut parser::v0::PartitionIndex, storage: &mut S) -> io::Result<usize>
	where
		S: io::Write + io::Seek,
	{
		let offset = storage.seek(io::SeekFrom::Current(0))?;
		if let Some(i) = index.index_uuid.get(&self.id) {
			let mut row = index.rows.get_mut(*i).unwrap();
			row.chunk_offset = offset as u64;
			row.chunk_size = 0;
			row.is_visible = self.is_visible();
			row.is_locked = self.is_locked();
			row.position = *self.position;
			row.name = String::from(&*self.note);
		} else {
			let row = parser::v0::PartitionTableRow {
				id: self.id,
				chunk_type: parser::v0::ChunkType::Note,
				chunk_offset: offset as u64,
				chunk_size: 0,
				is_root: false,
				is_visible: self.is_visible(),
				is_locked: self.is_locked(),
				is_folded: false,
				position: *self.position,
				size: Extent2::new(0, 0),
				name: String::from(&*self.note),
				children: Vec::new(),
				preview: Vec::new(),
			};
			index.index_uuid.insert(row.id, index.rows.len());
			index.rows.push(row);
		}
		Ok(0)
	}
}
