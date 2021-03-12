use serde::{Serialize, Deserialize};
use ffmpeg::{format::{context::Input, stream::Disposition}, Discard, media, Rational, codec};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Metadata {
	pub format: Format,
	pub best: Best,
	pub streams: Vec<Stream>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Format {
	pub name: String,
	pub description: String,
	pub extensions: Vec<String>,
	pub mime_types: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Best {
	pub video: Option<usize>,
	pub audio: Option<usize>,
	pub subtitle: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Stream {
	pub index: usize,
	pub time_base: Rational,
	pub start_time: i64,
	pub duration: i64,
	pub frames: i64,
	pub disposition: Disposition,
	pub discard: Discard,
	pub rate: Rational,
	pub avg_frame_rate: Rational,
	// TODO(meh): side_data

	pub codec: Codec,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Codec {
	pub id: codec::Id,
	pub name: String,
	pub description: String,

	pub data: Data,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Data {
	Audio(Audio),
	Video(Video),
	Subtitle(Subtitle),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Audio {
	pub bit_rate: usize,
	pub max_bit_rate: usize,
	pub delay: usize,
	pub rate: u32,
	pub channels: u16,
	pub format: ffmpeg::format::Sample,
	pub frames: usize,
	pub align: usize,
	pub channel_layout: ffmpeg::ChannelLayout,
	pub frame_start: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Video {
	pub bit_rate: usize,
	pub max_bit_rate: usize,
	pub delay: usize,
	pub width: u32,
	pub height: u32,
	pub format: ffmpeg::format::Pixel,
	pub has_b_frames: bool,
	pub aspect_ratio: ffmpeg::Rational,
	pub color_space: ffmpeg::color::Space,
	pub color_range: ffmpeg::color::Range,
	pub color_primaries: ffmpeg::color::Primaries,
	pub color_transfer_characteristic: ffmpeg::color::TransferCharacteristic,
	pub chroma_location: ffmpeg::chroma::Location,
	pub references: usize,
	pub intra_dc_precision: u8,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Subtitle {
}

impl Metadata {
	pub fn new(input: &Input) -> Self {
		let format = Format {
			name: input.format().name().into(),
			description: input.format().description().into(),
			extensions: input.format().extensions().into_iter().map(String::from).collect(),
			mime_types: input.format().mime_types().into_iter().map(String::from).collect(),
		};

		let best = Best {
			video: input.streams().best(media::Type::Video).map(|s| s.index()),
			audio: input.streams().best(media::Type::Audio).map(|s| s.index()),
			subtitle: input.streams().best(media::Type::Subtitle).map(|s| s.index()),
		};

		let streams = input.streams().into_iter().flat_map(|stream| {
			let codec = match stream.codec().medium() {
				media::Type::Audio => {
					let audio = stream.codec().decoder().audio().ok()?;

					Codec {
						id: audio.codec()?.id(),
						name: audio.codec()?.name().into(),
						description: audio.codec()?.description().into(),

						data: Data::Audio(Audio {
							bit_rate: audio.bit_rate(),
							max_bit_rate: audio.max_bit_rate(),
							delay: audio.delay(),
							rate: audio.rate(),
							channels: audio.channels(),
							format: audio.format(),
							frames: audio.frames(),
							align: audio.align(),
							channel_layout: audio.channel_layout(),
							frame_start: audio.frame_start(),
						})
					}
				}

				media::Type::Video => {
					let video = stream.codec().decoder().video().ok()?;

					Codec {
						id: video.codec()?.id(),
						name: video.codec()?.name().into(),
						description: video.codec()?.description().into(),

						data: Data::Video(Video {
							bit_rate: video.bit_rate(),
							max_bit_rate: video.max_bit_rate(),
							delay: video.delay(),
							width: video.width(),
							height: video.height(),
							format: video.format(),
							has_b_frames: video.has_b_frames(),
							aspect_ratio: video.aspect_ratio(),
							color_space: video.color_space(),
							color_range: video.color_range(),
							color_primaries: video.color_primaries(),
							color_transfer_characteristic: video.color_transfer_characteristic(),
							chroma_location: video.chroma_location(),
							references: video.references(),
							intra_dc_precision: video.intra_dc_precision(),
						})
					}
				}

				media::Type::Subtitle => {
					let subtitle = stream.codec().decoder().subtitle().ok()?;

					Codec {
						id: subtitle.codec()?.id(),
						name: subtitle.codec()?.name().into(),
						description: subtitle.codec()?.description().into(),

						data: Data::Subtitle(Subtitle {

						})
					}
				}

				_ => return None
			};

			Some(Stream {
				index: stream.index(),
				time_base: stream.time_base(),
				start_time: stream.start_time(),
				duration: stream.duration(),
				frames: stream.frames(),
				disposition: stream.disposition(),
				discard: stream.discard(),
				rate: stream.rate(),
				avg_frame_rate: stream.avg_frame_rate(),

				codec,
			})
		})
		.collect::<Vec<_>>();

		Metadata { format, best, streams }
	}
}
