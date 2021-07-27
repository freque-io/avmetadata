use std::env;
use avmetadata::Metadata;

fn main() {
  ffmpeg::init().unwrap();

  let input = ffmpeg::format::input(&env::args().nth(1).expect("missing file")).unwrap();
  println!("{:#?}", Metadata::new(&input));
}
