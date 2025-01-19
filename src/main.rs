mod dictionary;
mod theme;

use eframe::egui::{self, RichText};

use dictionary::{load_dictionary, Dictionary, Entry};
use theme::MOCHA;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Kaikki Kerralla",
        native_options,
        Box::new(|cc| Ok(Box::new(Kerralla::new(cc)))),
    )
}

struct Kerralla {
    dictionary: Dictionary,
    result: Vec<Entry>,
    query: String,
}

impl Kerralla {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // TODO figure out egui error handling or async loading -- for now we just hope we can load
        // and block the app creation until we do
        let dictionary = load_dictionary().unwrap();
        Self {
            dictionary,
            result: Vec::new(),
            query: String::new(),
        }
    }
}

impl eframe::App for Kerralla {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, move |ui| {
            let response = ui.add(egui::TextEdit::singleline(&mut self.query).lock_focus(true));
            response.request_focus();

            if response.changed() {
                self.result = self
                    .dictionary
                    .predictive_search(&self.query)
                    .take(20)
                    .map(|(_, v): (String, &Entry)| v)
                    .cloned()
                    .collect();
            }

            for entry in self.result.iter() {
                let section_color = match entry.pos.as_str() {
                    "noun" => MOCHA.red,
                    "adj" => MOCHA.green,
                    "verb" => MOCHA.yellow,
                    "adv" => MOCHA.blue,
                    _ => MOCHA.text,
                };
                ui.label(
                    RichText::new(format!("{}, {}", entry.word, entry.pos))
                        .heading()
                        .color(section_color)
                        .size(24.0),
                );
                for sense in entry.senses.iter() {
                    let label = sense
                        .glosses
                        .first()
                        .map_or("[No Definition]", |v| v);
                    ui.label(RichText::new(label).color(MOCHA.text).size(18.0));
                }
            }
        });
    }
}
