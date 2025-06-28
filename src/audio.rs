//! Audio decoding module
//!
use super::*;
use symphonia::core::audio::{AudioBufferRef, SampleBuffer, Signal};
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// Reads a file returns its buffer and a sample_rate
pub fn load_audio_file(path: &str) -> Result<(Vec<f32>, u32), Box<dyn Error>> {
    let src = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(src), Default::default());
    let hint = Hint::new();
    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();
    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio track found")?;

    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or("Unknown sample rate")?;
    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)?;

    let mut audio_buf = vec![];
    let mut sample_buf = None;

    loop {
        match format.next_packet() {
            Ok(packet) => {
                let buffer = decoder.decode(&packet)?;
                let mut mono_samples = to_mono_f32(&buffer, &mut sample_buf);
                audio_buf.append(&mut mono_samples);
            }
            Err(SymphoniaError::IoError(ref err))
                if err.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break
            }
            Err(err) => return Err(Box::new(err)),
        }
    }
    Ok((audio_buf, sample_rate))
}

fn to_mono_f32<'a>(
    buffer: &'a AudioBufferRef<'a>,
    sample_buf: &'a mut Option<SampleBuffer<f32>>,
) -> Vec<f32> {
    if sample_buf.is_none() {
        let spec = *(buffer.spec());
        let duration = buffer.capacity() as u64;
        let buf = SampleBuffer::<f32>::new(duration, spec);
        *sample_buf = Some(buf);
    }

    let s_buf = sample_buf.as_mut().unwrap();
    s_buf.copy_interleaved_ref(buffer.clone());
    let samples = s_buf.samples();
    let spec = buffer.spec();
    let mut mono_samples = Vec::with_capacity(buffer.frames());

    if spec.channels.count() == 1 {
        mono_samples.extend_from_slice(samples);
    } else {
        mono_samples.extend(
            samples
                .chunks_exact(spec.channels.count())
                .map(|chunk| chunk.iter().sum::<f32>() / spec.channels.count() as f32),
        );
    }
    mono_samples
}
