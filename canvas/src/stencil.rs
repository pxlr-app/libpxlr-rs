use crate::braille::braille_fmt2;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use color::*;
use serde::{Deserialize, Serialize};
use vek::{geom::repr_c::Rect, vec::repr_c::extent2::Extent2};

#[derive(Clone, Serialize, Deserialize)]
pub struct Stencil {
	pub rect: Rect<i32, i32>,
	pub mask: BitVec<Lsb0, u8>,
	pub channel: Channel,
	pub data: Vec<u8>,
}

impl Stencil {
	/// Create a new empty stencil
	pub fn new(size: Extent2<i32>, channel: Channel) -> Self {
		let len = (size.w * size.h) as usize;
		let mut buffer = Vec::with_capacity(len * channel.pixel_stride());
		let default_pixel = channel.default_pixel();
		for _ in 0..len {
			buffer.extend_from_slice(&default_pixel);
		}
		Self::from_buffer(size, channel, buffer)
	}

	/// Create a stencil from pixel data
	pub fn from_buffer(size: Extent2<i32>, channel: Channel, buffer: Vec<u8>) -> Self {
		let len = (size.w * size.h) as usize;
		assert_eq!(len * channel.pixel_stride(), buffer.len());
		let mut mask = bitvec![Lsb0, u8; 1; len];
		mask.set_uninitialized(false);
		Self {
			rect: Rect::new(0, 0, size.w, size.h),
			mask,
			channel,
			data: buffer,
		}
	}

	/// Create a stencil from pixel data and masking invisible one based on alpha
	pub fn from_buffer_mask_alpha(size: Extent2<i32>, channel: Channel, buffer: Vec<u8>) -> Self {
		match channel {
			Channel::Lumaa | Channel::LumaaNormal | Channel::Rgba | Channel::RgbaNormal => {
				let len = (size.w * size.h) as usize;
				let stride = channel.pixel_stride();
				assert_eq!(len * stride, buffer.len());
				let mut mask = bitvec![Lsb0, u8; 0; len];
				// #[cfg(feature = "rayon")]
				// let chunks = buffer.par_chunks(stride);
				// #[cfg(not(feature = "rayon"))]
				let chunks = buffer.chunks(stride);

				let data = chunks
					.enumerate()
					.filter_map(|(i, data)| {
						let pixel = Pixel::from_buffer(&data, channel);
						let alpha = match channel {
							Channel::Lumaa | Channel::LumaaNormal => pixel.lumaa().unwrap().alpha,
							Channel::Rgba | Channel::RgbaNormal => pixel.rgba().unwrap().alpha,
							_ => 0,
						};
						if alpha == 0 {
							None
						} else {
							mask.set(i, true);
							Some(data.to_vec())
						}
					})
					.flatten()
					.collect::<Vec<_>>();

				Self {
					rect: Rect::new(0, 0, size.w, size.h),
					mask,
					channel,
					data,
				}
			}
			_ => Self::from_buffer(size, channel, buffer),
		}
	}

	/// Try to retrieve a pixel at coordinate
	pub fn try_get(&self, x: i32, y: i32) -> Option<&[u8]> {
		// if self.rect.contains_point(Vec2::new(x, y)) {
		if self.rect.x <= x
			&& x < self.rect.x + self.rect.w
			&& self.rect.y <= y
			&& y < self.rect.y + self.rect.h
		{
			let index =
				(y.wrapping_sub(self.rect.y) * self.rect.w + x.wrapping_sub(self.rect.x)) as usize;
			self.try_index(index)
		} else {
			None
		}
	}

	/// Try to retrieve a pixel at index
	pub fn try_index(&self, index: usize) -> Option<&[u8]> {
		if self.mask[index] {
			let stride = self.channel.pixel_stride();
			let count: usize = self.mask[..index].count_ones();
			Some(&self.data[(count * stride)..((count + 1) * stride)])
		} else {
			None
		}
	}

	/// Merge two stencil and blend them together if need be
	pub fn merge(frt: &Self, bck: &Self, blend_mode: Blend, compose_op: Compose) -> Self {
		assert_eq!(frt.channel, bck.channel);
		let channel = frt.channel;

		// Calculate new size
		let rect = frt.rect.union(bck.rect);

		// Allocate new buffers
		let stride = frt.channel.pixel_stride();
		let mut mask = bitvec![Lsb0, u8; 0; (rect.w * rect.h) as usize];
		let mut data: Vec<u8> = Vec::with_capacity((rect.w * rect.h * stride as i32) as usize);
		let mut tmp = frt.channel.default_pixel();

		for i in 0..mask.len() {
			let x = (i % rect.w as usize) as i32 + rect.x;
			let y = (i / rect.w as usize) as i32 + rect.y;

			let frt_buf = frt.try_get(x, y);
			let bck_buf = bck.try_get(x, y);

			match (frt_buf, bck_buf) {
				(None, None) => mask.set(i, false),
				(Some(frt_buf), None) => {
					mask.set(i, true);
					data.extend_from_slice(frt_buf);
				}
				(None, Some(bck_buf)) => {
					mask.set(i, true);
					data.extend_from_slice(bck_buf);
				}
				(Some(frt_buf), Some(bck_buf)) => {
					mask.set(i, true);
					let frt_px = Pixel::from_buffer(frt_buf, frt.channel);
					let bck_px = Pixel::from_buffer(bck_buf, frt.channel);
					let mut pixel = PixelMut::from_buffer_mut(&mut tmp, channel);
					pixel
						.blend(blend_mode, compose_op, &frt_px, &bck_px)
						.unwrap();
					data.extend_from_slice(&tmp);
				}
			}
		}
		Self {
			rect,
			mask,
			channel,
			data,
		}
	}

	/// Iterate over pixel of this stencil
	pub fn iter(&self) -> StencilIterator {
		StencilIterator {
			bit_offset: 0,
			data_offset: 0,
			rect: self.rect,
			mask: &self.mask,
			pixel_stride: self.channel.pixel_stride(),
			data: &self.data,
		}
	}

	/// Iterate over pixel of this stencil
	pub fn iter_mut(&mut self) -> StencilMutIterator {
		StencilMutIterator {
			bit_offset: 0,
			data_offset: 0,
			rect: self.rect,
			mask: &self.mask,
			pixel_stride: self.channel.pixel_stride(),
			data: &mut self.data,
		}
	}
}

impl std::fmt::Debug for Stencil {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Stencil ( {} )",
			braille_fmt2(
				&self.mask,
				self.rect.w as usize,
				self.rect.h as usize,
				"\n           "
			)
		)
	}
}

impl std::ops::Add for &Stencil {
	type Output = Stencil;

	fn add(self, other: Self) -> Self::Output {
		Stencil::merge(self, other, Blend::Normal, Compose::Lighter)
	}
}

pub struct StencilIterator<'stencil> {
	bit_offset: usize,
	data_offset: usize,
	rect: Rect<i32, i32>,
	mask: &'stencil BitVec<Lsb0, u8>,
	pixel_stride: usize,
	data: &'stencil Vec<u8>,
}

impl<'stencil> Iterator for StencilIterator<'stencil> {
	type Item = (i32, i32, &'stencil [u8]);

	fn next(&mut self) -> Option<(i32, i32, &'stencil [u8])> {
		while self.bit_offset < self.mask.len() {
			let bit_offset = self.bit_offset;
			self.bit_offset += 1;
			let bit = self.mask[bit_offset];
			if bit {
				let x = bit_offset % self.rect.w as usize;
				let y = (bit_offset / self.rect.w as usize) | 0;
				self.data_offset += 1;
				return Some((
					x as i32 + self.rect.x,
					y as i32 + self.rect.y,
					&self.data[(self.data_offset.wrapping_sub(1) * self.pixel_stride)
						..(self.data_offset * self.pixel_stride)],
				));
			}
		}
		return None;
	}
}

impl<'stencil> IntoIterator for &'stencil Stencil {
	type Item = (i32, i32, &'stencil [u8]);
	type IntoIter = StencilIterator<'stencil>;

	fn into_iter(self) -> Self::IntoIter {
		self.iter()
	}
}

pub struct StencilMutIterator<'stencil> {
	bit_offset: usize,
	data_offset: usize,
	rect: Rect<i32, i32>,
	mask: &'stencil BitVec<Lsb0, u8>,
	pixel_stride: usize,
	data: &'stencil mut Vec<u8>,
}

impl<'stencil> Iterator for StencilMutIterator<'stencil> {
	type Item = (i32, i32, &'stencil mut [u8]);

	fn next<'iter>(&'iter mut self) -> Option<(i32, i32, &'stencil mut [u8])> {
		while self.bit_offset < self.mask.len() {
			let bit_offset = self.bit_offset;
			self.bit_offset += 1;
			let bit = self.mask[bit_offset];
			if bit {
				let x = bit_offset % self.rect.w as usize;
				let y = (bit_offset / self.rect.w as usize) | 0;
				self.data_offset += 1;
				let data: &'iter mut [u8] = &mut self.data[(self.data_offset.wrapping_sub(1)
					* self.pixel_stride)
					..(self.data_offset * self.pixel_stride)];
				let data =
					unsafe { std::mem::transmute::<&'iter mut [u8], &'stencil mut [u8]>(data) };
				return Some((x as i32 + self.rect.x, y as i32 + self.rect.y, data));
			}
		}
		return None;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_from_buffer() {
		let s = Stencil::from_buffer(Extent2::new(2, 2), Channel::Luma, vec![1u8, 2, 3, 4]);
		assert_eq!(*s.mask, bitvec![1, 1, 1, 1]);
		assert_eq!(*s.data, [1u8, 2, 3, 4]);
	}

	#[test]
	fn test_from_buffer_mask_alpha() {
		let s = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1u8, 255, 0, 0, 0, 0, 4, 1],
		);
		assert_eq!(*s.mask, bitvec![1, 0, 0, 1]);
		assert_eq!(*s.data, [1u8, 255, 4, 1]);
	}

	#[test]
	fn test_debug() {
		let s = Stencil::new(Extent2::new(3, 1), Channel::Luma);
		assert_eq!(format!("{:?}", s), "Stencil ( ⠉⠁ )");
		let s = Stencil::new(Extent2::new(1, 3), Channel::Luma);
		assert_eq!(format!("{:?}", s), "Stencil ( ⠇ )");
	}

	#[test]
	fn test_merge() {
		let a = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1, 255, 0, 0, 0, 0, 4, 255],
		);
		assert_eq!(format!("{:?}", a), "Stencil ( ⠑ )");
		let b = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![0, 0, 2, 255, 3, 255, 0, 0],
		);
		assert_eq!(format!("{:?}", b), "Stencil ( ⠊ )");
		let c = Stencil::merge(&a, &b, Blend::Normal, Compose::Lighter);
		assert_eq!(format!("{:?}", c), "Stencil ( ⠛ )");
		assert_eq!(c.data, vec![1, 255, 2, 255, 3, 255, 4, 255]);

		let a = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1, 255, 2, 255, 0, 0, 4, 255],
		);
		assert_eq!(format!("{:?}", a), "Stencil ( ⠙ )");
		let b = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![0, 0, 20, 255, 3, 255, 0, 0],
		);
		assert_eq!(format!("{:?}", b), "Stencil ( ⠊ )");
		let c = Stencil::merge(&a, &b, Blend::Normal, Compose::Lighter);
		assert_eq!(format!("{:?}", c), "Stencil ( ⠛ )");
		assert_eq!(c.data, vec![1, 255, 2, 255, 3, 255, 4, 255]);

		let a = Stencil::from_buffer_mask_alpha(
			Extent2::new(1, 2),
			Channel::Lumaa,
			vec![1, 255, 2, 255],
		);
		assert_eq!(format!("{:?}", a), "Stencil ( ⠃ )");
		let mut b = Stencil::from_buffer_mask_alpha(
			Extent2::new(1, 2),
			Channel::Lumaa,
			vec![3, 255, 4, 255],
		);
		b.rect.x = 2;
		assert_eq!(format!("{:?}", b), "Stencil ( ⠃ )");
		let c = Stencil::merge(&a, &b, Blend::Normal, Compose::Lighter);
		assert_eq!(format!("{:?}", c), "Stencil ( ⠃⠃ )");
		assert_eq!(c.data, vec![1, 255, 3, 255, 2, 255, 4, 255]);
	}

	#[test]
	fn iter() {
		let a = Stencil::from_buffer(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1, 255, 2, 255, 3, 255, 4, 255],
		);
		let pixels: Vec<_> = a
			.iter()
			.map(|(_, _, data)| data.to_vec())
			.flatten()
			.collect();
		assert_eq!(pixels, vec![1, 255, 2, 255, 3, 255, 4, 255]);

		let a = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1, 255, 0, 0, 0, 0, 4, 255],
		);
		let pixels: Vec<_> = a
			.iter()
			.map(|(_, _, data)| data.to_vec())
			.flatten()
			.collect();
		assert_eq!(pixels, vec![1, 255, 4, 255]);
	}

	#[test]
	fn iter_mut() {
		let mut a = Stencil::from_buffer(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1, 255, 2, 255, 3, 255, 4, 255],
		);
		let pixels: Vec<_> = a
			.iter_mut()
			.map(|(_, _, data)| data.to_vec())
			.flatten()
			.collect();
		assert_eq!(pixels, vec![1, 255, 2, 255, 3, 255, 4, 255]);

		let mut a = Stencil::from_buffer_mask_alpha(
			Extent2::new(2, 2),
			Channel::Lumaa,
			vec![1, 255, 0, 0, 0, 0, 4, 255],
		);
		let pixels: Vec<_> = a
			.iter_mut()
			.map(|(_, _, data)| data.to_vec())
			.flatten()
			.collect();
		assert_eq!(pixels, vec![1, 255, 4, 255]);
	}
}
