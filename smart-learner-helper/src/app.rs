use rodio::{Decoder, OutputStream, Source};
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;
use std::thread;

use smart_learner_core::{card::Card, deck::Deck, field::Field, result::Result};

use crate::{
    config::Config,
    data::{self, DeckFromFile},
};

pub struct App {
    pub config: Config,
    pub decks: Vec<DeckFromFile>,
    pub current_deck: usize,
    current_card: Option<usize>,
    pub card_front: String,
    pub card_back: String,
    pub search_text: String,
    pub back_search: bool,
}

impl App {
    pub fn new() -> Self {
        let config: Config = confy::load("smart-learner", None).unwrap();
        let decks = data::fetch_decks(&Path::new(&config.folder_path));
        Self {
            config,
            decks,
            current_deck: 0,
            current_card: None,
            card_front: String::new(),
            card_back: String::new(),
            search_text: String::new(),
            back_search: false,
        }
    }

    pub fn new_deck(&mut self, deck_name: String) {
        let folder_path = Path::new(&self.config.folder_path);
        let path = folder_path.join(Path::new(&deck_name));
        self.decks.push(DeckFromFile {
            value: Deck::new(deck_name),
            path: path.as_path().to_str().unwrap().to_string() + ".sdeck",
        });
    }

    /// Returns (card_exists, got a new card).
    pub fn get_card_for_revision(&mut self) -> (bool, bool) {
        self.current_card = match self.current_card {
            Some(result) => {
                if self.decks[self.current_deck].value.cards[result].current_repeat_in == 0 {
                    return (true, false);
                } else {
                    self.decks[self.current_deck].value.due_card()
                }
            }
            None => self.decks[self.current_deck].value.due_card(),
        };

        if self.current_card.is_some() {
            self.change_card(self.current_card.unwrap());
            (true, true)
        } else {
            (false, false)
        }
    }

    pub fn get_answer(&self) -> String {
        if self.current_card.is_some() {
            self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
                .back
                .text
                .clone()
        } else {
            "".to_string()
        }
    }

    pub fn get_question(&self) -> String {
        if self.current_card.is_some() {
            self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
                .front
                .text
                .clone()
        } else {
            "".to_string()
        }
    }

    pub fn current_deck_name(&self) -> String {
        if self.decks.len() > self.current_deck {
            self.decks[self.current_deck].value.name.clone()
        } else {
            "No decks".to_string()
        }
    }

    pub fn create_card(&mut self) -> bool {
        if self.decks.len() == 0 {
            self.current_card = None;
            return false;
        }

        self.decks[self.current_deck].value.cards.push(Card::new(
            Field {
                text: "New front".to_string(),
                audio_path: None,
            },
            Field {
                text: "New back".to_string(),
                audio_path: None,
            },
        ));
        self.change_card(self.decks[self.current_deck].value.cards.len() - 1);
        true
    }

    pub fn edit_card(&mut self) {
        self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
            .front
            .text = self.card_front.clone();

        self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
            .back
            .text = self.card_back.clone();
    }

    pub fn search(&mut self) -> Vec<(usize, String)> {
        if self.decks.len() == 0 {
            return Vec::new();
        }

        self.decks[self.current_deck]
            .value
            .search(self.back_search, self.search_text.clone())
    }

    pub fn change_card(&mut self, card_index: usize) {
        self.current_card = Some(card_index);
        let card = &self.decks[self.current_deck].value.cards[self.current_card.unwrap()];
        self.card_front = card.front.text.clone();
        self.card_back = card.back.text.clone();
    }

    pub fn card_revised(&mut self, result: Result) {
        self.decks[self.current_deck].value.cards[self.current_card.unwrap()].review(result);
    }

    pub fn delete_card(&mut self) {
        self.decks[self.current_deck]
            .value
            .cards
            .swap_remove(self.current_card.unwrap());
        self.current_card = None;
    }

    fn play_audio(&self, path: String) {
        let path = Path::new(&self.config.folder_path)
            .to_path_buf()
            .join(Path::new("audio"))
            .join(Path::new(&path));

        thread::spawn(|| {
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();

            let (_stream, stream_handle) = OutputStream::try_default().unwrap();

            stream_handle.play_raw(source.convert_samples()).unwrap();
            std::thread::sleep(std::time::Duration::from_secs(5));
        });
    }

    pub fn play_front_audio(&self) {
        let card = &self.decks[self.current_deck].value.cards[self.current_card.unwrap()];
        if card.front.audio_path.is_some() {
            self.play_audio(card.front.audio_path.clone().unwrap());
        }
    }

    pub fn play_back_audio(&self) {
        let card = &self.decks[self.current_deck].value.cards[self.current_card.unwrap()];
        if card.back.audio_path.is_some() {
            self.play_audio(card.back.audio_path.clone().unwrap());
        }
    }

    fn get_audio_file(&mut self, path: String) {
        // Getting a file name
        let old_file_name = Path::new(&path)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        // Audio folder in folder with decks
        let audio_path_stem = Path::new(&self.config.folder_path).join("audio");

        // Path to a audio folder and old filename
        let mut new_file_path = audio_path_stem.join(old_file_name.clone());

        // Change filename if it exists
        if audio_path_stem.join(&old_file_name).exists() {
            for i in 1.. {
                let clone = old_file_name.clone();
                let mut path_iter = clone.split('.');
                let file_extention = ".".to_owned() + path_iter.next_back().unwrap();
                let file_name = path_iter.next_back().unwrap();
                let file_name_string = format!("{}{}{}", file_name, i.to_string(), file_extention);
                let file_name_path = Path::new(&file_name_string);
                new_file_path = audio_path_stem.join(file_name_path);

                if !new_file_path.exists() {
                    break;
                }
            }
        }

        // Copy a file to the local folder
        fs::copy(path.clone(), new_file_path).unwrap();
    }

    pub fn change_front_audio(&mut self, path: String) {
        self.get_audio_file(path.clone());
        self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
            .front
            .audio_path = Some(path);
    }

    pub fn change_back_audio(&mut self, path: String) {
        self.get_audio_file(path.clone());
        self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
            .back
            .audio_path = Some(path);
    }

    pub fn front_audio_exists(&self) -> bool {
        self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
            .front
            .audio_path
            .is_some()
    }

    pub fn back_audio_exists(&self) -> bool {
        self.decks[self.current_deck].value.cards[self.current_card.unwrap()]
            .back
            .audio_path
            .is_some()
    }
}
