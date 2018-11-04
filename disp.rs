use ::std::io::*;
use ::std::io;
use ::std::fs::File;

fn to_u32_be(a: [u8; 4]) -> u32 {
    unsafe { ::std::mem::transmute::<[u8; 4], u32>(a) }.to_be()
}

/// http://mpgedit.org/mpgedit/mpeg_format/mpeghdr.htm
/// Returns frame length
fn read_header(fp: &mut File, verbose: bool) -> io::Result<u64> {
    let mut buf = [0u8; 4];
    fp.read(&mut buf)?;
    let header = to_u32_be(buf);
    let frame_sync = header >> 21;
    if frame_sync != 0x7ff {
        eprintln!("Frame sync is not set!!");
    }
    let audio_version_id = (header >> 19) & 3;
    let layer_description = (header >> 17) & 3;
    let protection = (header >> 16) & 1;
    let bitrate_index = (header >> 12) & 0xf;
    // Allows only V1, L3
    let bitrate = [0, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320][bitrate_index as usize];
    let sampling_rate_freq_index = (header >> 10) & 3;
    let sampling_rate_freq_table =
        [44100, 48000, 32000];
    let sampling_factor = [4, 0, 2, 1][audio_version_id as usize];
    let sample_rate = sampling_rate_freq_table[sampling_rate_freq_index as usize] / sampling_factor;
    let padding = (header >> 9) & 1;
    let channel_mode = (header >> 6) & 3;
    let copyright = (header >> 3) & 1;
    let frame_length = 144 * bitrate * 1000 / sample_rate + padding;
    if verbose {
        println!("{}", ["MPEG Version 2.5", "reserved", "MPEG Version 2", "MPEG Version 1"][audio_version_id as usize]);
        println!("{}", ["reserved", "Layer III", "Layer II", "Layer I"][layer_description as usize]);
        println!("{}", if protection == 0 { "Protected by CRC" } else { "Not protected" });
        println!("bitrate = {}kbps", bitrate);
        println!("Sampling rate freq = {}Hz", sample_rate);
        println!("Channel Mode: {}", ["Stereo", "Joint stereo", "Dual channel", "Single channel"][channel_mode as usize]);
        println!("Audio is {}copyrighted", if copyright == 0 { "not " } else { "" });
        println!("frame length = 0x{:06x} bytes", frame_length);
    }
    Ok(frame_length.into())
}

fn main() -> io::Result<()> {
    let args: Vec<_> = ::std::env::args().collect();
    if args.len() <= 1 {
        eprintln!("usage: {} FILENAME.mp3", args[0]);
        return Err(Error::new(ErrorKind::Other, "provide arguments"));
    }
    let filename = args[1].clone();
    let mut fp = File::open(&filename)?;

    fp.seek(SeekFrom::End(0))?;
    let filesize = fp.seek(SeekFrom::Current(0))?;
    let mut current = 0;
    let mut frame_size = 0;
    while current < filesize {
        fp.seek(SeekFrom::Start(current))?;
        let frame_length = read_header(&mut fp, false)?;
        current += frame_length;
        frame_size += 1;
    }
    println!("frame_size = {}", frame_size);
    Ok(())
}
