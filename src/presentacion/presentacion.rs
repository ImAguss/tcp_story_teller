use crossterm::cursor::{self, MoveTo};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use std::io::stdout;

use crate::dominio::visual::PasoTCP;

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
    loop {
        let (ancho_terminal, alto_terminal) = crossterm::terminal::size()?;
        let (ancho_bloque, alto_bloque) = diapositivas[indice].dimensiones();
        let x = (ancho_terminal.saturating_sub(ancho_bloque) / 2);
        let y = (alto_terminal.saturating_sub(alto_bloque) / 2);
        execute!(stdout(), Clear(ClearType::All))?;

        for (i, linea) in diapositivas[indice].bloque_completo().lines().enumerate() {
            execute!(stdout(), MoveTo(x, y + i as u16))?;
            print!("{}", linea);
        }

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Left => {
                        if indice > 0 {
                            indice -= 1
                        }
                    }
                    KeyCode::Right => {
                        if indice > max_len {
                            break;
                        } else {
                            indice += 1;
                        }
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
