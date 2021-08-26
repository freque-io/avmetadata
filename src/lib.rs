use std::collections::HashMap;
use ffmpeg::{
	codec,
	format::{context::Input, stream::Disposition},
	media, Discard, Rational,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Metadata {
	pub format: Format,
	pub best: Best,
	pub streams: Vec<Stream>,
	pub details: HashMap<String, String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Format {
	pub name: String,
	pub aliases: Vec<String>,
	pub description: String,
	pub extensions: Vec<String>,
	pub mime_types: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Best {
	pub audio: Option<usize>,
	pub video: Option<usize>,
	pub subtitle: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Stream {
	pub index: usize,
	pub time_base: Rational,
	pub start_time: Option<i64>,
	pub duration: Option<i64>,
	pub frames: i64,
	pub disposition: Disposition,
	pub discard: Discard,
	pub frame_rate: Rational,
	pub avg_frame_rate: Rational,
	// TODO(meh): side_data
	pub content: Content,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Codec {
	pub id: codec::Id,
	pub name: String,
	pub description: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum Content {
	Unknown(Unknown),
	Audio(Audio),
	Video(Video),
	Data(Data),
	Subtitle(Subtitle),
	Attachment(Attachment),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Unknown;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Audio {
	pub codec: Codec,
	pub bit_rate: usize,
	pub max_bit_rate: usize,
	pub delay: usize,
	pub sample_rate: u32,
	pub channels: u16,
	pub format: ffmpeg::format::Sample,
	pub frames: usize,
	pub align: usize,
	pub channel_layout: ffmpeg::ChannelLayout,
	pub frame_start: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Video {
	pub codec: Codec,
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
pub struct Data {}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Subtitle {
	pub codec: Codec,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Attachment {}

impl Metadata {
	pub fn new(input: &Input) -> ffmpeg::Result<Self> {
		let format = Format {
			name: input.format().name().split(",").next().unwrap().into(),
			aliases: input.format().name().split(",").skip(1).map(String::from).collect(),
			description: input.format().description().into(),
			extensions: input
				.format()
				.extensions()
				.into_iter()
				.map(String::from)
				.collect(),
			mime_types: input
				.format()
				.mime_types()
				.into_iter()
				.map(String::from)
				.collect(),
		};

		let best = Best {
			video: input.streams().best(media::Type::Video).map(|s| s.index()),
			audio: input.streams().best(media::Type::Audio).map(|s| s.index()),
			subtitle: input
				.streams()
				.best(media::Type::Subtitle)
				.map(|s| s.index()),
		};

		let streams = input
			.streams()
			.into_iter()
			.map(|stream| {
				let content = match stream.codec().medium() {
					media::Type::Unknown => {
						Content::Unknown(Unknown)
					}

					media::Type::Audio => {
						let audio = stream.codec().decoder().audio()?;

						Content::Audio(Audio {
							codec: Codec {
								id: audio.codec().ok_or(ffmpeg::Error::Bug)?.id(),
								name: audio.codec().ok_or(ffmpeg::Error::Bug)?.name().into(),
								description: audio.codec().ok_or(ffmpeg::Error::Bug)?.description().into(),
							},
							bit_rate: audio.bit_rate(),
							max_bit_rate: audio.max_bit_rate(),
							delay: audio.delay(),
							sample_rate: audio.sample_rate(),
							channels: audio.channels(),
							format: audio.format(),
							frames: audio.frames(),
							align: audio.align(),
							channel_layout: audio.channel_layout(),
							frame_start: audio.frame_start(),
						})
					}

					media::Type::Video => {
						let video = stream.codec().decoder().video()?;

						Content::Video(Video {
							codec: Codec {
								id: video.codec().ok_or(ffmpeg::Error::Bug)?.id(),
								name: video.codec().ok_or(ffmpeg::Error::Bug)?.name().into(),
								description: video.codec().ok_or(ffmpeg::Error::Bug)?.description().into(),
							},
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

					media::Type::Data => {
						Content::Data(Data { })
					}

					media::Type::Subtitle => {
						let subtitle = stream.codec().decoder().subtitle()?;

						Content::Subtitle(Subtitle {
							codec: Codec {
								id: subtitle.codec().ok_or(ffmpeg::Error::Bug)?.id(),
								name: subtitle.codec().ok_or(ffmpeg::Error::Bug)?.name().into(),
								description: subtitle.codec().ok_or(ffmpeg::Error::Bug)?.description().into(),
							},
						})
					}

					media::Type::Attachment => {
						Content::Attachment(Attachment { })
					}
				};

				Ok(Stream {
					index: stream.index(),
					time_base: stream.time_base(),
					start_time: stream.start_time(),
					duration: stream.duration(),
					frames: stream.frames(),
					disposition: stream.disposition(),
					discard: stream.discard(),
					frame_rate: stream.frame_rate(),
					avg_frame_rate: stream.avg_frame_rate(),
					content,
				})
			})
			.collect::<ffmpeg::Result<Vec<_>>>()?;

		let details = input.metadata().iter().map(|(a, b)| (a.into(), b.into())).collect();

		Ok(Metadata {
			format,
			best,
			streams,
			details,
		})
	}
}
