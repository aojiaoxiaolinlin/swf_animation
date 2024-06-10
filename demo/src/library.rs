use std::collections::HashMap;

use swf::CharacterId;

use crate::{character::Character, graphic::Graphic};

pub struct MovieLibrary {
    characters: HashMap<CharacterId, Character>,
    pub instance_count: u16,
}
impl MovieLibrary {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            instance_count: 1,
        }
    }
    pub fn register_character(&mut self, id: CharacterId, character: Character) {
        self.characters.insert(id, character);
    }
    pub fn character(&self, id: CharacterId) -> Option<&Character> {
        self.characters.get(&id)
    }

    pub fn get_graphic(&self, id: CharacterId) -> Option<Graphic> {
        if let Some(Character::Graphic(graphic)) = self.characters.get(&id).clone() {
            Some(graphic.clone())
        } else {
            None
        }
    }
}
