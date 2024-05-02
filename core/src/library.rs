use std::{collections::HashMap, sync::{Arc, Weak}};

use swf::CharacterId;
use weak_table::PtrWeakKeyHashMap;

use crate::{character::Character, tag_utils::SwfMovie};

pub struct Library {
    movie_libraries:PtrWeakKeyHashMap<Weak<SwfMovie>, MovieLibrary>,
}


pub struct MovieLibrary {
    swf: Arc<SwfMovie>,
    characters: HashMap<CharacterId, Character>,
    jpeg_tables: Option<Vec<u8>>,
    // fonts: FontMap,
}