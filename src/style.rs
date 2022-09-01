use crate::DataEvent;

use super::DataEntity;
use crate::ui::details::Success;
use bevy::render::color::Color;

#[derive(Clone)]
pub struct ExStyle {
	pub color: Color,
}

impl Hash for ExStyle {
	fn hash<H: Hasher>(&self, state: &mut H) {
		(self.color.r() as u32).hash(state);
		(self.color.g() as u32).hash(state);
		(self.color.b() as u32).hash(state);
	}
}

impl Eq for ExStyle {}
impl PartialEq for ExStyle {
	fn eq(&self, other: &Self) -> bool {
		self.color == other.color
	}
}

use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
};
fn calculate_hash<T: Hash>(t: &T) -> u64 {
	let mut s = DefaultHasher::new();
	t.hash(&mut s);
	s.finish()
}

use palette::FromColor;

// coloring block timestamp actually
pub fn color_block_number(block_number: i64, darkside: bool) -> Color {
	let color = palette::Lchuv::new(
		if darkside { 40. } else { 80. },
		80. + (block_number % 100) as f32,
		(block_number % 360) as f32,
	);
	let rgb: palette::rgb::Srgb = palette::rgb::Srgb::from_color(color);
	Color::rgba(rgb.red, rgb.green, rgb.blue, 0.7)
}

pub fn style_event(entry: &DataEntity) -> ExStyle {
	let darkside = entry.details().doturl.is_darkside();
	let msg = crate::content::is_message(entry);
	match entry {
		DataEntity::Event(data_event @ DataEvent { .. }) => style_data_event(data_event),
		// match event.pallet.as_str() {
		//     "Staking" => ExStyle {
		//         color: Color::hex("00ffff").unwrap(),
		//     },
		//     "Deposit" => ExStyle {
		//         color: Color::hex("e6007a").unwrap(),
		//     },
		//     "Withdraw" => ExStyle {
		//         color: Color::hex("e6007a").unwrap(),
		//     },
		//     _ => ExStyle {
		//         color: Color::hex("000000").unwrap(),
		//     },
		// }
		DataEntity::Extrinsic { details, .. } => {
			let color = palette::Lchuv::new(
				if darkside { 40. } else { 80. },
				80. + (calculate_hash(&details.variant) as f32 % 100.),
				(calculate_hash(&details.pallet) as f32) % 360.,
			);
			let rgb: palette::rgb::Srgb = palette::rgb::Srgb::from_color(color);

			ExStyle { color: Color::rgba(rgb.red, rgb.green, rgb.blue, if msg { 0.5 } else { 1. }) }
		},
	}
}

pub fn style_data_event(entry: &DataEvent) -> ExStyle {
	let darkside = entry.details.doturl.is_darkside();

	// let msg = crate::content::is_event_message(entry);
	let raw = &entry.details;
	if matches!(
		(raw.pallet.as_str(), raw.variant.as_str()),
		("System", "ExtrinsicFailed") /* | ("PolkadotXcm", "Attempted") - only an error if
		                               * !completed variant. */
	) || entry.details.success == Success::Sad
	{
		return ExStyle { color: Color::rgb(1., 0., 0.) }
	}
	if entry.details.success == Success::Worried {
		return ExStyle {
			// Trump Orange
			color: Color::rgb(1., 0.647_058_84, 0.),
		}
	}

	let color = palette::Lchuv::new(
		if darkside { 40. } else { 80. },
		80. + (calculate_hash(&raw.variant) as f32 % 100.),
		(calculate_hash(&raw.pallet) as f32) % 360.,
	);
	let rgb: palette::rgb::Srgb = palette::rgb::Srgb::from_color(color);

	// println!("rgb {} {} {}", rgb.red, rgb.green, rgb.blue);

	ExStyle { color: Color::rgb(rgb.red, rgb.green, rgb.blue) }
}
