use std::error::Error;
use std::{io, thread};
use std::sync::mpsc;
use std::time::Duration;
use crossterm::{event, ExecutableCommand, terminal};
use crossterm::cursor::{Hide, Show};
use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use space_invaders::{frame, render};
use space_invaders::frame::new_frame;

fn main() -> Result<(), Box<dyn Error>> {

    //terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?; // this is similar to vim taking u to an  alternate screen
    stdout.execute(Hide)?;

    // render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);

        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame, false);
            last_frame = curr_frame;
        }
    });

    //game loop
    'gameloop: loop {
        //per frame init
        let curr_frame = new_frame();


        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }
        // draw and render
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }


    // clean up
    drop(render_tx); // newer rust detects and auto drops - non required lifecycles
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
