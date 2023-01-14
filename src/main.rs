mod audio;

use std::{error::Error, time::Duration, sync::mpsc, thread, io};
use audio::Audio;
use crossterm::{
  terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, self
  },
  ExecutableCommand,
  cursor::{
    Hide, Show
  }, event::{
    self, Event, KeyCode
  }
};
use invaders::{frame::{self, new_frame}, render};

fn main()  -> Result <(), Box<dyn Error>> {
  let mut audio = Audio::new();
  audio.add("explode", "audios/bomb.ogg");
  audio.add("lose", "audios/lose.ogg");
  audio.add("move", "audios/move.wav");
  audio.add("pew", "audios/arrow_hit.ogg");
  audio.add("startup", "audios/startup.mp3");
  audio.add("win", "audios/achieved.ogg");
  audio.play("move");

  let mut stdout = std::io::stdout();
  terminal::enable_raw_mode()?;
  stdout.execute(EnterAlternateScreen)?;
  stdout.execute(Hide)?;

	let (render_tx, render_rx) = mpsc::channel();
	let render_handle = thread::spawn(move || {
    let mut last_frame = frame::new_frame();
    let mut stdout = io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);

    loop {
      let current_frame = match render_rx.recv() {
        Ok(x) => x,
        Err(_) => break,
      };

      render::render(&mut stdout, &last_frame, &current_frame, false);
      last_frame = current_frame;
    }
  });

  'gameLoop: loop {
    let curr_frame = new_frame();
    while event::poll(Duration::default())? {
      if let Event::Key(key_event) = event::read()? {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                audio.play("lose");
                break 'gameLoop
            },
            _ => {}
        }
      }
    }

    // Draw and render
    let _ = render_tx.send(curr_frame);
    thread::sleep(Duration::from_millis(1));
  }

  drop(render_tx);
  render_handle.join().unwrap();
  audio.wait();
  stdout.execute(Show)?;
  stdout.execute(LeaveAlternateScreen)?;
  terminal::disable_raw_mode()?;
  Ok(())
}