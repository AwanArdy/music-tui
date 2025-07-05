use image::load_from_memory;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use viuer::Config;

// Impor App dari modul player
use crate::player::{App, PlaybackState};

pub fn ui(frame: &mut Frame, app: &mut App) {
    // --- Layout ---
    // Layout utama dengan panel Now Playing di atas dan panel Durasi di bawah
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Panel Now Playing
            Constraint::Min(0),    // Area utama (playlist, cover, dll)
            Constraint::Length(3), // Panel Durasi
        ])
        .split(frame.size());

    let now_playing_pane = main_chunks[0];
    let middle_pane = main_chunks[1];
    let duration_pane = main_chunks[2];

    // Layout untuk area utama (kiri dan kanan)
    let top_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
        .split(middle_pane);

    let left_pane = top_chunks[0];
    let right_pane = top_chunks[1];

    let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(15), Constraint::Min(10)])
        .split(right_pane);

    let cover_pane = right_layout[0];
    let metadata_pane = right_layout[1];

    // --- Widget Now Playing (Atas) ---
    let now_playing_block = Block::default().borders(Borders::ALL).title("Now Playing");
    let (status_text, status_style) = match app.playback_state {
        PlaybackState::Playing => ("[Playing]", Style::default().fg(Color::Green)),
        PlaybackState::Paused => ("[Paused]", Style::default().fg(Color::Yellow)),
        PlaybackState::Stopped => ("[Stopped]", Style::default().fg(Color::Red)),
    };

    let song_title = if let Some(index) = app.playlist_state.selected() {
        if let Some(track) = app.playlist.get(index) {
            track.title.as_str()
        } else {
            "-"
        }
    } else {
        "-"
    };

    let now_playing_text = vec![Line::from(vec![
        Span::styled(status_text, status_style),
        Span::raw(format!(" {}", song_title)),
    ])];

    let now_playing_widget = Paragraph::new(now_playing_text)
        .block(now_playing_block)
        .alignment(Alignment::Center);
    frame.render_widget(now_playing_widget, now_playing_pane);


    // --- Widget Playlist (Kiri) ---
    if app.playlist.is_empty() {
        let empty_playlist_msg = Paragraph::new(
            "
Tidak ada file musik (.flac) yang ditemukan.

Tekan 'q' untuk keluar.",
        )
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Playlist"))
        .alignment(Alignment::Center);
        frame.render_widget(empty_playlist_msg, left_pane);
    } else {
        // Ambil judul dari setiap Track
        let playlist_items: Vec<ListItem> = app
            .playlist
            .iter()
            .map(|track| ListItem::new(track.title.as_str()))
            .collect();

        let playlist_widget = List::new(playlist_items)
            .block(Block::default().borders(Borders::ALL).title("Playlist"))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .fg(Color::Black)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");
        frame.render_stateful_widget(playlist_widget, left_pane, &mut app.playlist_state);
    }

    // --- Widget Album Cover (Kanan Atas) ---
    let cover_block = Block::default().borders(Borders::ALL).title("Cover");
    let inner_cover_area = cover_block.inner(cover_pane);
    frame.render_widget(cover_block, cover_pane);

    let cover_picture = app.playlist_state.selected()
        .and_then(|index| app.playlist.get(index))
        .and_then(|track| track.cover_art.as_ref());

    if let Some(picture) = cover_picture {
        if let Ok(img) = load_from_memory(&picture.data) {
            let config = Config {
                x: inner_cover_area.x,
                y: inner_cover_area.y as i16,
                width: Some(inner_cover_area.width as u32),
                height: Some(inner_cover_area.height as u32),
                ..Default::default()
            };
            let _ = viuer::print(&img, &config);
        }
    } else {
        let placeholder_text = Paragraph::new("Cover Art
Not Found")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder_text, inner_cover_area);
    }

    // --- Widget Metadata (Kanan Bawah) ---
    let metadata_content = if let Some(selected_index) = app.playlist_state.selected() {
        // Akses track langsung dari playlist
        if let Some(track) = app.playlist.get(selected_index) {
            vec![
                Line::from(Span::styled("Title  : ", Style::default().bold())),
                // Akses field .title dari struct Track
                Line::from(Span::raw(&track.title)),
                Line::from(""),
                Line::from(Span::styled("Artist : ", Style::default().bold())),
                // Akses field .artist dari struct Track
                Line::from(Span::raw(track.artist.as_deref().unwrap_or("N/A"))),
                Line::from(""),
                Line::from(Span::styled("Album  : ", Style::default().bold())),
                // Akses field .album dari struct Track
                Line::from(Span::raw(track.album.as_deref().unwrap_or("N/A"))),
                Line::from(""),
                Line::from(Span::styled("Year   : ", Style::default().bold())),
                // Akses field .year dari struct Track
                Line::from(Span::raw(track.year.as_deref().unwrap_or("N/A"))),
            ]
        } else {
            vec![Line::from("Lagu tidak ditemukan.")]
        }
    } else {
        vec![Line::from("Tidak ada lagu dalam playlist.")]
    };

    let metadata_widget = Paragraph::new(metadata_content)
        .wrap(Wrap { trim: true })
        .block(Block::default().borders(Borders::ALL).title("Metadata"));
    frame.render_widget(metadata_widget, metadata_pane);

    // --- Widget Duration (Bawah) ---
    let duration_block = Block::default().borders(Borders::ALL).title("Duration");

    let duration_text = if let Some(selected_index) = app.playlist_state.selected() {
        if let Some(track) = app.playlist.get(selected_index) {
            let current_pos = app.playback_position;
            let total_duration = track.duration.unwrap_or_default();

            let format_duration = |d: std::time::Duration| -> String {
                let secs = d.as_secs();
                format!("{:02}:{:02}", secs / 60, secs % 60)
            };

            Line::from(format!(
                "{} / {}",
                format_duration(current_pos),
                format_duration(total_duration)
            ))
        } else {
            Line::from("00:00 / 00:00")
        }
    } else {
        Line::from("00:00 / 00:00")
    };

    let duration_widget = Paragraph::new(duration_text)
        .block(duration_block)
        .alignment(Alignment::Center);
    frame.render_widget(duration_widget, duration_pane);

    // --- Widget Hint Shortcut (Bawah) ---
    let shuffle_status = if app.shuffle_enabled { "On" } else { "Off" };
    let hint_text = vec![Line::from(vec![
        Span::styled("Esc", Style::default().bold()),
        Span::raw(": Quit | "),
        Span::styled("q/e", Style::default().bold()),
        Span::raw(": Prev/Next | "),
        Span::styled("Enter", Style::default().bold()),
        Span::raw(": Play | "),
        Span::styled("Space", Style::default().bold()),
        Span::raw(": Pause | "),
        Span::styled("s", Style::default().bold()),
        Span::raw(format!(": Shuffle ({})", shuffle_status)),
    ])];
    let hint_widget = Paragraph::new(hint_text).alignment(Alignment::Center);
    frame.render_widget(hint_widget, frame.size());
}