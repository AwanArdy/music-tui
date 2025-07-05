mod file;
mod player;
mod ui;

use std::{
    io::{self, stdout},
    time::Duration,
};

use crossterm::{
    ExecutableCommand,
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

use ratatui::prelude::*;
use rfd::FileDialog;

use crate::file::scan_music_folder;
use crate::player::App;

fn main() -> io::Result<()> {
    println!("Membuka dialog pemilihan folder...");
    let folder_path = FileDialog::new()
        .set_title("Pilih Folder Musik")
        .pick_folder();

    let playlist = if let Some(path) = folder_path {
        println!("Memindai folder: {:?}", path);
        match scan_music_folder(path) {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Gagal memindai folder: {}", e);
                Vec::new()
            }
        }
    } else {
        println!("Folder tidak ditemukan");
        return Ok(());
    };

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new(playlist);

    while app.running {
        app.update_playback(); // Perbarui status pemutaran
        terminal.draw(|frame| ui::ui(frame, &mut app))?;

        if event::poll(Duration::from_millis(50))? { // Kurangi waktu polling
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc => app.running = false,
                        KeyCode::Char('e') | KeyCode::Down => app.next_item(),
                        KeyCode::Char('q') | KeyCode::Up => app.previous_item(),
                        KeyCode::Enter => app.play_selected(),
                        KeyCode::Char(' ') => app.toggle_playback(),
                        KeyCode::Char('s') => app.toggle_shuffle(),
                        _ => {}
                    }
                }
            }
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
