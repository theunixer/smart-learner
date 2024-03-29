#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{
    egui::{self, Id, Key},
    epaint::Vec2,
};
use egui_file::FileDialog;
use smart_learner_core::result::Result;
use smart_learner_helper::app::App;

fn main() {
    env_logger::init();
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(640.0, 480.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Smart learner",
        options,
        Box::new(|_cc| Box::<GuiApp>::default()),
    )
    .unwrap();
}

struct GuiApp {
    app: App,
    state: GuiState,
    new_deck_name: String,
    file_dialog: Option<FileDialog>,
}

enum GuiState {
    Main,
    Browser,
    NewCard,
    Editor,
    RevisingWithoutAnswer,
    RevisingWithAnswer,
    Settings,
}

impl Default for GuiApp {
    fn default() -> Self {
        Self {
            app: App::new(),
            state: GuiState::Main,
            new_deck_name: "".to_string(),
            file_dialog: None,
        }
    }
}

impl eframe::App for GuiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Showing the page
        match self.state {
            GuiState::Main => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    // Thingy to create new decks
                    ui.horizontal(|ui| {
                        let label = ui.label("Deck name:");
                        ui.text_edit_singleline(&mut self.new_deck_name)
                            .labelled_by(label.id);
                        let button = ui.button("Create deck");
                        if button.clicked() {
                            self.app.new_deck(self.new_deck_name.clone());
                            self.new_deck_name = String::new();
                        }
                    });

                    // Displaying decks
                    egui::containers::ScrollArea::vertical().show(ui, |ui| {
                        for (index, deck) in self.app.decks.iter().enumerate() {
                            if ui.link(&deck.value.name).clicked() {
                                self.state = GuiState::RevisingWithoutAnswer;
                                self.app.current_deck = index;
                            }
                        }
                    });
                });
            }

            GuiState::Editor => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.group(|ui| {
                        let label = ui.label("Front:");
                        ui.text_edit_multiline(&mut self.app.card_front)
                            .labelled_by(label.id);

                        if ui.button("Choose audio").clicked() {
                            let mut dialog =
                                FileDialog::open_file(None).default_size(Vec2::new(480.0, 300.0));
                            dialog.open();
                            self.file_dialog = Some(dialog);
                        }

                        if let Some(dialog) = &mut self.file_dialog {
                            if dialog.show(ctx).selected() {
                                if let Some(file) = dialog.path() {
                                    self.app.change_front_audio(
                                        file.as_path().to_str().unwrap().to_string(),
                                    );
                                }
                            }
                        }
                    });

                    ui.group(|ui| {
                        let label = ui.label("Back:");
                        ui.text_edit_multiline(&mut self.app.card_back)
                            .labelled_by(label.id);

                        if ui.button("Choose audio").clicked() {
                            let mut dialog =
                                FileDialog::open_file(None).default_size(Vec2::new(480.0, 300.0));
                            dialog.open();
                            self.file_dialog = Some(dialog);
                        }

                        if let Some(dialog) = &mut self.file_dialog {
                            if dialog.show(ctx).selected() {
                                if let Some(file) = dialog.path() {
                                    self.app.change_back_audio(
                                        file.as_path().to_str().unwrap().to_string(),
                                    );
                                }
                            }
                        }
                    });

                    if ui.button("Save").clicked() {
                        self.app.edit_card();
                        self.state = GuiState::Main;
                    }

                    if ui.button("Delete").clicked() {
                        self.app.delete_card();
                        self.state = GuiState::Main;
                    }
                });
            }

            GuiState::Browser => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        //choose the deck
                        egui::ComboBox::from_label("Deck")
                            .selected_text(self.app.current_deck_name())
                            .show_ui(ui, |ui| {
                                for (index, deck) in self.app.decks.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut self.app.current_deck,
                                        index,
                                        &deck.value.name,
                                    );
                                }
                            });

                        //search field
                        ui.text_edit_singleline(&mut self.app.search_text);

                        //front or back
                        ui.checkbox(&mut self.app.back_search, "Back search");
                    });
                    //search results
                    egui::containers::ScrollArea::vertical().show(ui, |ui| {
                        for entry in self.app.search() {
                            ui.group(|ui| {
                                if ui.link(entry.1).clicked() {
                                    self.app.change_card(entry.0);
                                    self.state = GuiState::Editor;
                                }
                            });
                        }
                    });
                });
            }

            GuiState::RevisingWithoutAnswer => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let revision_result = self.app.get_card_for_revision();

                    if revision_result.0 {
                        if revision_result.1 {
                            self.app.play_front_audio();
                        }

                        ui.group(|ui| {
                            ui.heading(&self.app.card_front);
                            if self.app.front_audio_exists() {
                                if ui.button("Play audio").clicked() {
                                    self.app.play_front_audio();
                                }
                            }
                        });

                        ui.horizontal(|ui| {
                            if ui.button("Show answer").clicked()
                                || ctx.input(|i| i.key_pressed(Key::Space))
                            {
                                self.state = GuiState::RevisingWithAnswer;
                                self.app.play_back_audio();
                            }
                            if ui.button("Edit").clicked() {
                                self.state = GuiState::Editor;
                            }
                        });
                    } else {
                        ui.heading("No cards to review.");
                    }
                });
            }

            GuiState::RevisingWithAnswer => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.group(|ui| {
                        ui.heading(&self.app.card_front);
                        if self.app.front_audio_exists() {
                            if ui.button("Play audio").clicked() {
                                self.app.play_front_audio();
                            }
                        }
                    });

                    ui.group(|ui| {
                        ui.heading(&self.app.card_back);
                        if self.app.back_audio_exists() {
                            if ui.button("Play audio").clicked() {
                                self.app.play_back_audio();
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        let mut result = None;

                        if ui.button("Wrong").clicked() {
                            result = Some(Result::Wrong);
                        }

                        if ui.button("Difficult").clicked() {
                            result = Some(Result::Difficult);
                        }

                        if ui.button("Easy").clicked() {
                            result = Some(Result::Easy);
                        }

                        if result.is_some() {
                            self.app.card_revised(result.unwrap());
                            self.state = GuiState::RevisingWithoutAnswer;
                        }
                    })
                });
            }

            GuiState::NewCard => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("Add a card");
                    //Displaying decks.
                    egui::ComboBox::from_label("Deck")
                        .selected_text(self.app.current_deck_name())
                        .show_ui(ui, |ui| {
                            for (index, deck) in self.app.decks.iter().enumerate() {
                                ui.selectable_value(
                                    &mut self.app.current_deck,
                                    index,
                                    &deck.value.name,
                                );
                            }
                        });

                    if ui.button("Create").clicked() {
                        if self.app.create_card() {
                            self.state = GuiState::Editor;
                        }
                    }
                });
            }

            GuiState::Settings => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if ui.button("Change folder with decks").clicked() {
                        let mut dialog =
                            FileDialog::select_folder(None).default_size(Vec2::new(480.0, 300.0));
                        dialog.open();
                        self.file_dialog = Some(dialog);
                    }

                    if let Some(dialog) = &mut self.file_dialog {
                        if dialog.show(ctx).selected() {
                            if let Some(file) = dialog.path() {
                                self.app.config.folder_path =
                                    file.as_path().to_str().unwrap().to_string();
                            }
                        }
                    }
                });
            }
        }
        // Menu
        egui::TopBottomPanel::bottom(Id::new("menu")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Home").clicked() {
                    self.state = GuiState::Main;
                };
                if ui.button("Browse cards").clicked() {
                    self.state = GuiState::Browser;
                };
                if ui.button("New card").clicked() {
                    self.state = GuiState::NewCard;
                };
                if ui.button("Settings").clicked() {
                    self.state = GuiState::Settings;
                };
            });
        });
    }
}
