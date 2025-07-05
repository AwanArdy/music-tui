use metaflac::{Tag, block::Picture};
use std::{fs, io, path::PathBuf, time::Duration};

// Jadikan struct dan field-nya public
#[derive(Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub year: Option<String>,
    pub cover_art: Option<Picture>,
    pub duration: Option<Duration>,
}

pub fn scan_music_folder(path: PathBuf) -> io::Result<Vec<Track>> {
    let mut tracks = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "flac") {
            if let Ok(tag) = Tag::read_from_path(&path) {
                let stream_info = tag.get_streaminfo();

                let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
                let title = tag
                    .get_vorbis("TITLE")
                    .and_then(|mut v| v.next())
                    .unwrap_or(&file_stem)
                    .to_string();

                let artist = tag.get_vorbis("ARTIST").and_then(|mut v| v.next().map(String::from));
                let album = tag.get_vorbis("ALBUM").and_then(|mut v| v.next().map(String::from));
                let year = tag.get_vorbis("DATE").and_then(|mut v| v.next().map(String::from));
                let cover_art = tag.pictures().next().cloned();

                let duration = stream_info.map(|s| {
                    let seconds = s.total_samples as f64 / s.sample_rate as f64;
                    Duration::from_secs_f64(seconds)
                });

                tracks.push(Track {
                    path,
                    title: title.to_string(),
                    artist,
                    album,
                    year,
                    cover_art,
                    duration,
                });
            }
        }
    }
    tracks.sort_by(|a, b| a.title.cmp(&b.title));
    Ok(tracks)
}
