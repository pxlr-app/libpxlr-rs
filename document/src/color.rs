use crate::file::reader;
use crate::file::writer::Writer;
use math::blend::*;
use math::Lerp;
use nom::number::complete::{le_f32, le_u8};
use nom::IResult;
use num_traits::identities::Zero;
use serde::{Deserialize, Serialize};
use std::default::Default;
use std::io::Write;
use std::ops::{Add, Div, Mul, Sub};

pub trait Color: Copy {}

macro_rules! define_colors {
	{$(
		$color:ident ($($name:ident:$type:ty:$reader:ident),+);
	)+} => {

		#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
		pub enum ColorMode {
			$($color),+
		}

		$(
			#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
			pub struct $color {
				$(pub $name: $type),+
			}

			impl $color {
				pub fn new($($name: $type),+) -> $color {
					$color { $($name: $name,)+ }
				}
			}

			impl Color for $color {}

			impl Default for $color {
				fn default() -> Self {
					$color { $($name: <$type as Zero>::zero(),)+ }
				}
			}

			impl Add for $color {
				type Output = $color;

				fn add(self, other: Self) -> Self {
					$color { $($name: ((self.$name as f64) + (other.$name as f64)) as $type,)+ }
				}
			}

			impl Sub for $color {
				type Output = $color;

				fn sub(self, other: Self) -> Self {
					$color { $($name: ((self.$name as f64) - (other.$name as f64)) as $type,)+ }
				}
			}

			impl Mul for $color {
				type Output = $color;

				fn mul(self, other: Self) -> Self {
					$color { $($name: ((self.$name as f64) * (other.$name as f64)) as $type,)+ }
				}
			}

			impl Div for $color {
				type Output = $color;

				fn div(self, other: Self) -> Self {
					$color { $($name: ((self.$name as f64) / (other.$name as f64)) as $type,)+ }
				}
			}

			impl Mul<$color> for f32 {
				type Output = $color;

				fn mul(self, other: $color) -> Self::Output {
					$color { $($name: (self * (other.$name as f32)) as $type,)+ }
				}
			}

			impl Mul<f32> for $color {
				type Output = $color;

				fn mul(self, other: f32) -> Self::Output {
					$color { $($name: ((self.$name as f32) * other) as $type,)+ }
				}
			}

			impl Blend for $color {
				type Output = $color;

				fn blend(from: &Self, to: &Self, mode: &BlendMode) -> Self {
					match mode {
						BlendMode::Normal => *to,
						BlendMode::Add => *from + *to,
						BlendMode::Subtract => *from - *to,
						BlendMode::Multiply => *from * *to,
						BlendMode::Divide => *from / *to,
						_ => *to,
					}
				}
			}

			impl Lerp<f32> for $color {
				type Output = $color;

				fn lerp_unclamped(from: Self, to: Self, factor: f32) -> Self::Output {
					from + (to - from) * factor
				}
			}

			impl reader::v0::Reader for $color {
				fn from_bytes(bytes: &[u8]) -> IResult<&[u8], $color> {
					$(
						let (bytes, $name) = $reader(bytes)?;
					)+
					Ok((bytes, $color { $($name),+ }))
				}
			}
			impl Writer for $color {
				fn write_to<W: Write>(&self, writer: &mut W) -> std::io::Result<usize> {
					let mut b: usize = 0;
					$(
						b += writer.write(&self.$name.to_le_bytes())?;
					)+
					Ok(b)
				}
			}
		)+
	}
}

define_colors! {
	I (i:u8:le_u8);
	IXYZ (i:u8:le_u8, x:f32:le_f32, y:f32:le_f32, z:f32:le_f32);
	UV (u:f32:le_f32, v:f32:le_f32);
	RGB (r:u8:le_u8, g:u8:le_u8, b:u8:le_u8);
	RGBA (r:u8:le_u8, g:u8:le_u8, b:u8:le_u8, a:u8:le_u8);
	RGBAXYZ (r:u8:le_u8, g:u8:le_u8, b:u8:le_u8, a:u8:le_u8, x:f32:le_f32, y:f32:le_f32, z:f32:le_f32);
}
