use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use swf::CharacterId;
use weak_table::PtrWeakKeyHashMap;

use crate::{character::Character, tag_utils::SwfMovie};

pub struct Library {
    movie_libraries: PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary>,
}
impl Library {
    pub fn empty() -> Self {
        Library {
            movie_libraries: PtrWeakKeyHashMap::new(),
        }
    }
    pub fn library_for_movie_mut(&mut self, movie: Arc<SwfMovie>) -> &mut MovieLibrary {
        self.movie_libraries
            .entry(movie.clone())
            .or_insert_with(|| MovieLibrary::new(movie))
    }
}
pub struct MovieLibrary {
    characters: HashMap<CharacterId, Character>,
    jpeg_tables: Option<Vec<u8>>,
    swf: Arc<SwfMovie>,
}

impl MovieLibrary {
    pub fn new(swf: Arc<SwfMovie>) -> Self {
        Self {
            characters: HashMap::new(),
            jpeg_tables: None,
            swf,
        }
    }
    pub fn register_character(&mut self, id: CharacterId, character: Character) {
        if !self.contains_character(id) {
            self.characters.insert(id, character);
        }else {
            dbg!("Character already exists");
        }
    }
    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }
    pub fn character_by_id(&self, id: CharacterId) -> Option<&Character> {
        self.characters.get(&id)
    }
}
