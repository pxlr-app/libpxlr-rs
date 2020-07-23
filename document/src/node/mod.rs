use crate::prelude::*;
use math::{Extent2, Vec2};
use nom::number::complete::le_u16;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use uuid::Uuid;
mod group;
mod note;
mod palette;
mod sprite;
pub use group::*;
pub use note::*;
pub use palette::*;
pub use sprite::*;

pub trait Node: Any + Debug + patch::Patchable {
	fn id(&self) -> Uuid;
}
impl Downcast for dyn Node {}

pub type NodeRef = Arc<NodeType>;
pub type NodeList = Vec<NodeRef>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
	Note(Note),
	Group(Group),
	Sprite(Sprite),
	Palette(Palette),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeKind {
	Note,
	Group,
	Sprite,
	Palette,
}

impl parser::Parse for NodeKind {
	fn parse(bytes: &[u8]) -> nom::IResult<&[u8], NodeKind> {
		let (bytes, idx) = le_u16(bytes)?;
		match idx {
			0 => Ok((bytes, NodeKind::Group)),
			1 => Ok((bytes, NodeKind::Note)),
			2 => Ok((bytes, NodeKind::Sprite)),
			3 => Ok((bytes, NodeKind::Palette)),
			_ => Err(nom::Err::Error((bytes, nom::error::ErrorKind::NoneOf))),
		}
	}
}

impl parser::Write for NodeKind {
	fn write(&self, writer: &mut dyn io::Write) -> io::Result<usize> {
		let idx: u16 = match self {
			NodeKind::Group => 0,
			NodeKind::Note => 1,
			NodeKind::Sprite => 2,
			NodeKind::Palette => 3,
		};
		writer.write_all(&idx.to_le_bytes())?;
		Ok(2)
	}
}

impl NodeType {
	pub fn as_node(&self) -> &dyn Node {
		match self {
			NodeType::Note(node) => node,
			NodeType::Group(node) => node,
			NodeType::Sprite(node) => node,
			NodeType::Palette(node) => node,
		}
	}
	pub fn as_documentnode(&self) -> Option<&dyn DocumentNode> {
		match self {
			NodeType::Note(node) => Some(node),
			NodeType::Group(node) => Some(node),
			NodeType::Sprite(node) => Some(node),
			NodeType::Palette(node) => Some(node),
			// _ => None,
		}
	}
	pub fn as_spritenode(&self) -> Option<&dyn SpriteNode> {
		match self {
			NodeType::Sprite(node) => Some(node),
			_ => None,
		}
	}
}

impl parser::v0::ParseNode for NodeType {
	fn parse_node<'bytes>(
		row: &parser::v0::IndexRow,
		items: NodeList,
		dependencies: NodeList,
		bytes: &'bytes [u8],
	) -> parser::Result<&'bytes [u8], NodeRef> {
		match row.chunk_type {
			NodeKind::Note => {
				<Note as parser::v0::ParseNode>::parse_node(row, items, dependencies, bytes)
			}
			NodeKind::Group => {
				<Group as parser::v0::ParseNode>::parse_node(row, items, dependencies, bytes)
			}
			NodeKind::Sprite => {
				<Sprite as parser::v0::ParseNode>::parse_node(row, items, dependencies, bytes)
			}
			NodeKind::Palette => {
				<Palette as parser::v0::ParseNode>::parse_node(row, items, dependencies, bytes)
			}
		}
	}
}

impl parser::v0::WriteNode for NodeType {
	fn write_node<W: io::Write + io::Seek>(
		&self,
		writer: &mut W,
		rows: &mut Vec<parser::v0::IndexRow>,
		dependencies: &mut Vec<NodeRef>,
	) -> io::Result<usize> {
		match self {
			NodeType::Note(node) => node.write_node(writer, rows, dependencies),
			NodeType::Group(node) => node.write_node(writer, rows, dependencies),
			NodeType::Sprite(node) => node.write_node(writer, rows, dependencies),
			NodeType::Palette(node) => node.write_node(writer, rows, dependencies),
		}
	}
}

pub trait Name {
	fn name(&self) -> String {
		"".into()
	}
	fn rename(&self, _name: String) -> Option<patch::PatchPair> {
		None
	}
}
pub trait Position {
	fn position(&self) -> Vec2<u32> {
		Vec2::new(0, 0)
	}
	fn translate(&self, _target: Vec2<u32>) -> Option<patch::PatchPair> {
		None
	}
}
pub trait Size {
	fn size(&self) -> Extent2<u32> {
		Extent2::new(0, 0)
	}
	fn resize(&self, _target: Extent2<u32>) -> Option<patch::PatchPair> {
		None
	}
}
pub trait Visible {
	fn visible(&self) -> bool {
		true
	}
	fn set_visibility(&self, _visible: bool) -> Option<patch::PatchPair> {
		None
	}
}
pub trait Locked {
	fn locked(&self) -> bool {
		false
	}
	fn set_lock(&self, _locked: bool) -> Option<patch::PatchPair> {
		None
	}
}
pub trait Folded {
	fn folded(&self) -> bool {
		false
	}
	fn set_fold(&self, _folded: bool) -> Option<patch::PatchPair> {
		None
	}
}

pub trait DocumentNode: Node + Name + Position + Size + Visible + Locked + Folded {}
impl Downcast for dyn DocumentNode {}

pub trait SpriteNode: Node + Name + Position + Size + Visible + Locked + Folded {
	fn color_mode(&self) -> ColorMode;
}
impl Downcast for dyn SpriteNode {}
