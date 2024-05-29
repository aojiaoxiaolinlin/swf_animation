use std::{borrow::Cow, cell::RefCell, collections::HashMap, rc::Rc};

use ruffle_render::utils::remove_invalid_jpeg_data;
use swf::CharacterId;

use crate::{
    character::Character,
    display_object::{
        movie_clip::{self, MovieClip},
        DisplayObject,
    },
};

pub struct MovieLibrary {
    characters: HashMap<CharacterId, Character>,
    jpeg_tables: Option<Vec<u8>>,
}

impl MovieLibrary {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            jpeg_tables: None,
        }
    }
    pub fn register_character(&mut self, id: CharacterId, character: Character) {
        if !self.contains_character(id) {
            self.characters.insert(id, character);
        } else {
            dbg!("Character already exists");
        }
    }
    pub fn contains_character(&self, id: CharacterId) -> bool {
        self.characters.contains_key(&id)
    }
    pub fn character_by_id(&mut self, id: CharacterId) -> Option<&Character> {
        self.characters.get(&id)
    }

    pub fn jpeg_tables(&self) -> Option<&[u8]> {
        self.jpeg_tables.as_ref().map(|data| &data[..])
    }
    pub fn set_jpeg_tables(&mut self, data: &[u8]) {
        if self.jpeg_tables.is_some() {
            dbg!("SWF contains multiple JPEGTables tags");
            return;
        }
        self.jpeg_tables = if data.is_empty() {
            None
        } else {
            Some(remove_invalid_jpeg_data(data).to_vec())
        };
    }
    pub fn characters(&self) -> &HashMap<CharacterId, Character> {
        &self.characters
    }
    pub fn instantiate_by_id(&self, id: CharacterId) -> Result<DisplayObject, Cow<'_, str>> {
        if let Some(character) = self.characters.get(&id) {
            self.instantiate_display_object(id, character)
        } else {
            dbg!("Character id does't exist in the library");
            Err("Character id doesn't exist".into())
        }
    }
    pub fn instantiate_display_object(
        &self,
        id: CharacterId,
        character: &Character,
    ) -> Result<DisplayObject, Cow<'static, str>> {
        match character {
            Character::MovieClip(movie_clip) => Ok(movie_clip.clone().into()),
            _ => Err("Not a DisplayObject".into()),
        }
    }
    pub fn length(&self) -> usize {
        self.characters.len()
    }
}
