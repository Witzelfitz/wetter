use crate::{
    api,
    data::{Bericht, Ort},
};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::{io, sync::mpsc, thread, time::Duration};

pub enum Antwort {
    Orte(Vec<Ort>),
    Wetter(Box<Bericht>),
}
pub type Nachricht = (u64, Result<Antwort, String>);
#[derive(Default)]
pub struct Eingabe {
    pub text: String,
    pub cursor: usize,
}
impl Eingabe {
    fn position(&self) -> usize {
        self.text
            .char_indices()
            .nth(self.cursor)
            .map_or(self.text.len(), |(i, _)| i)
    }
    fn taste(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char(c) if !c.is_control() => {
                self.text.insert(self.position(), c);
                self.cursor += 1;
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.text.chars().count()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.text.chars().count(),
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.position());
            }
            KeyCode::Delete if self.cursor < self.text.chars().count() => {
                self.text.remove(self.position());
            }
            _ => {}
        }
    }
}
pub enum Dialog {
    Suche(Eingabe),
    Orte(Vec<Ort>, usize),
    Hilfe,
}

pub struct App {
    pub bericht: Option<Bericht>,
    pub ansicht: usize,
    pub stunde: usize,
    pub dialog: Option<Dialog>,
    pub status: String,
    pub fehler: bool,
    pub laedt: bool,
    pub interaktiv: bool,
    pub offset: u16,
    nummer: u64,
    sender: mpsc::Sender<Nachricht>,
    empfaenger: mpsc::Receiver<Nachricht>,
}
impl App {
    pub fn neu(bericht: Option<Bericht>, interaktiv: bool) -> Self {
        let (sender, empfaenger) = mpsc::channel();
        Self {
            bericht,
            ansicht: 0,
            stunde: 0,
            dialog: None,
            status: String::new(),
            fehler: false,
            laedt: false,
            interaktiv,
            offset: 0,
            nummer: 0,
            sender,
            empfaenger,
        }
    }
    fn starten(
        &mut self,
        status: String,
        aufgabe: impl FnOnce() -> Result<Antwort, String> + Send + 'static,
    ) {
        self.nummer += 1;
        self.status = status;
        self.fehler = false;
        self.laedt = true;
        let nummer = self.nummer;
        let sender = self.sender.clone();
        thread::spawn(move || {
            let _ = sender.send((nummer, aufgabe()));
        });
    }
    pub fn suchen(&mut self, name: String) {
        self.dialog = None;
        self.starten(format!("Suche nach {name} …"), move || {
            api::suchen(&api::client()?, &name).map(Antwort::Orte)
        });
    }
    fn laden(&mut self, ort: Ort) {
        self.dialog = None;
        self.starten(format!("Aktualisiere {} …", ort.kurz()), move || {
            let wetter = api::wetter(&api::client()?, &ort)?;
            Ok(Antwort::Wetter(Box::new(Bericht { ort, wetter })))
        });
    }
    // Alte Antworten dürfen eine neuere Ortsauswahl nicht überschreiben.
    pub fn uebernehmen(&mut self, (nummer, antwort): Nachricht) {
        if nummer != self.nummer {
            return;
        }
        self.laedt = false;
        match antwort {
            Ok(Antwort::Orte(mut orte)) if orte.len() == 1 => self.laden(orte.remove(0)),
            Ok(Antwort::Orte(orte)) => {
                self.dialog = Some(Dialog::Orte(orte, 0));
                self.status.clear();
            }
            Ok(Antwort::Wetter(bericht)) => {
                self.bericht = Some(*bericht);
                self.status.clear();
                self.stunde = 0;
                self.offset = 0;
            }
            Err(fehler) => {
                self.fehler = true;
                self.status = format!("Aktualisierung fehlgeschlagen: {fehler}");
            }
        }
    }
    pub fn taste(&mut self, key: KeyEvent) -> bool {
        if key.modifiers.contains(KeyModifiers::CONTROL)
            && matches!(key.code, KeyCode::Char('c' | 'd'))
        {
            return false;
        }
        if let Some(dialog) = self.dialog.take() {
            match dialog {
                Dialog::Suche(mut input) => match key.code {
                    KeyCode::Esc => {
                        if self.bericht.is_none() {
                            return false;
                        }
                    }
                    KeyCode::Enter => {
                        if input.text.trim().chars().count() >= 2 {
                            self.suchen(input.text.trim().to_owned());
                        } else {
                            self.status = "Bitte mindestens zwei Zeichen eingeben.".into();
                            self.dialog = Some(Dialog::Suche(input));
                        }
                    }
                    _ => {
                        input.taste(key.code);
                        self.dialog = Some(Dialog::Suche(input));
                    }
                },
                Dialog::Orte(orte, mut index) => match key.code {
                    KeyCode::Esc => self.dialog = Some(Dialog::Suche(Eingabe::default())),
                    KeyCode::Enter => {
                        if let Some(ort) = orte.get(index) {
                            self.laden(ort.clone());
                        }
                    }
                    _ => {
                        if key.code == KeyCode::Up {
                            index = index.saturating_sub(1);
                        }
                        if key.code == KeyCode::Down {
                            index = (index + 1).min(orte.len().saturating_sub(1));
                        }
                        self.dialog = Some(Dialog::Orte(orte, index));
                    }
                },
                Dialog::Hilfe => {}
            }
            return true;
        }
        let anzahl = self
            .bericht
            .as_ref()
            .map_or(0, |b| b.wetter.stunden_indices().len());
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return false,
            KeyCode::Char('/') => self.dialog = Some(Dialog::Suche(Eingabe::default())),
            KeyCode::Char('?') => self.dialog = Some(Dialog::Hilfe),
            KeyCode::Char('r') if !self.laedt => {
                if let Some(b) = &self.bericht {
                    self.laden(b.ort.clone());
                }
            }
            KeyCode::Tab => {
                self.ansicht = (self.ansicht + 1) % 3;
                self.offset = 0;
            }
            KeyCode::BackTab => {
                self.ansicht = (self.ansicht + 2) % 3;
                self.offset = 0;
            }
            KeyCode::Char('1'..='3') => {
                if let KeyCode::Char(c) = key.code {
                    self.ansicht = c as usize - '1' as usize;
                    self.offset = 0;
                }
            }
            KeyCode::Left if self.ansicht == 1 => self.stunde = self.stunde.saturating_sub(1),
            KeyCode::Right if self.ansicht == 1 => {
                self.stunde = (self.stunde + 1).min(anzahl.saturating_sub(1))
            }
            KeyCode::Left => {
                self.ansicht = (self.ansicht + 2) % 3;
                self.offset = 0;
            }
            KeyCode::Right => {
                self.ansicht = (self.ansicht + 1) % 3;
                self.offset = 0;
            }
            KeyCode::Up => self.offset = self.offset.saturating_sub(1),
            KeyCode::Down => self.offset = self.offset.saturating_add(1).min(40),
            _ => {}
        }
        true
    }
}

pub fn tui(name: Option<String>) -> io::Result<()> {
    ratatui::run(|terminal| {
        let mut app = App::neu(None, true);
        if let Some(name) = name {
            app.suchen(name);
        } else {
            app.dialog = Some(Dialog::Suche(Eingabe::default()));
        }
        let mut neu_zeichnen = true;
        loop {
            while let Ok(nachricht) = app.empfaenger.try_recv() {
                app.uebernehmen(nachricht);
                neu_zeichnen = true;
            }
            if neu_zeichnen {
                terminal.draw(|frame| crate::ui::zeichnen(frame, &app))?;
                neu_zeichnen = false;
            }
            if event::poll(Duration::from_millis(80))? {
                match event::read()? {
                    Event::Key(key) if key.kind == KeyEventKind::Press => {
                        if !app.taste(key) {
                            break;
                        }
                        neu_zeichnen = true;
                    }
                    Event::Resize(_, _) => neu_zeichnen = true,
                    _ => {}
                }
            }
        }
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fehler_und_veraltete_antworten_behalten_die_bisherigen_daten() {
        let mut app = App::neu(Some(crate::test_support::beispiel()), true);
        app.nummer = 4;
        app.laedt = true;
        app.uebernehmen((3, Err("veralteter Fehler".into())));
        assert!(app.laedt);
        assert!(!app.fehler);
        app.uebernehmen((4, Err("offline".into())));
        assert!(!app.laedt);
        assert!(app.fehler);
        assert_eq!(app.bericht.as_ref().unwrap().ort.name, "Schönengrund");
        assert!(app.status.contains("offline"));
    }

    #[test]
    fn ein_ortswechsel_uebernimmt_ort_und_wetter_gemeinsam() {
        let mut bern = crate::test_support::beispiel();
        bern.ort.name = "Bern".into();
        let mut gallen = bern.clone();
        gallen.ort.name = "St. Gallen".into();
        gallen.wetter.current.temperature_2m = Some(7.0);
        let mut app = App::neu(Some(bern.clone()), true);
        app.nummer = 2;
        app.uebernehmen((2, Ok(Antwort::Wetter(Box::new(gallen)))));
        app.uebernehmen((1, Ok(Antwort::Wetter(Box::new(bern)))));
        let bericht = app.bericht.unwrap();
        assert_eq!(bericht.ort.name, "St. Gallen");
        assert_eq!(bericht.wetter.current.temperature_2m, Some(7.0));
    }
    #[test]
    fn stundenwahl_ist_begrenzt_und_suche_akzeptiert_unicode() {
        let mut app = App::neu(Some(crate::test_support::beispiel()), true);
        app.ansicht = 1;
        for _ in 0..40 {
            app.taste(KeyEvent::new(KeyCode::Right, KeyModifiers::NONE));
        }
        assert_eq!(app.stunde, 24);
        app.taste(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
        assert_eq!(app.ansicht, 2);
        let mut input = Eingabe::default();
        for c in "Zürich".chars() {
            input.taste(KeyCode::Char(c));
        }
        input.taste(KeyCode::Left);
        input.taste(KeyCode::Backspace);
        assert_eq!(input.text, "Zürih");
    }
}
