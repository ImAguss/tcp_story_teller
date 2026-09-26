use crossterm::cursor::{self, MoveTo};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::stdout;

use crate::Dominio::visual::PasoTCP;

pub struct HandleTerminal;
impl Drop for HandleTerminal {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), LeaveAlternateScreen, cursor::Show);
        let _ = disable_raw_mode();
    }
}

pub fn ejecutar_presentacion(diapositivas: Vec<PasoTCP>) -> std::io::Result<()> {
    let mut indice: usize = 0;
    let max_len = diapositivas.len() + 1;
    let _handle = HandleTerminal;

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, cursor::Hide)?;
    execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
    println!(
        "{}",
        diapositivas[indice].bloque_completo().replace('\n', "\r\n")
    );
    while indice < max_len {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Left => {
                        if indice > 0 {
                            execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
                            println!(
                                "{}",
                                diapositivas[indice - 1]
                                    .bloque_completo()
                                    .replace('\n', "\r\n")
                            );
                            indice -= 1;
                        }
                    }
                    KeyCode::Right => {
                        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
                        println!(
                            "{}",
                            diapositivas[indice + 1]
                                .bloque_completo()
                                .replace('\n', "\r\n")
                        );
                        indice += 1;
                    }
                    KeyCode::Enter => {
                        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0))?;
                        println!(
                            "{}",
                            diapositivas[indice + 1]
                                .bloque_completo()
                                .replace('\n', "\r\n")
                        );
                        indice += 1;
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}
