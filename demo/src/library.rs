use std::collections::HashMap;

use swf::CharacterId;

use crate::character::Character;

pub struct MovieLibrary {
    characters: HashMap<CharacterId, Character>,
}
impl MovieLibrary {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
        }
    }
    pub fn register_character(&mut self, id: CharacterId, character: Character) {
        self.characters.insert(id, character);
    }
    pub fn character(&self, id: CharacterId) -> Option<&Character> {
        self.characters.get(&id)
    }
    
}