
use std::sync::{Arc, Mutex};

use swf::error::Error;
use url::Url;

use crate::{tag_utils::SwfMovie, Player};

pub struct Loader{}

impl Loader {
    pub fn root_movie_loader(url:&Url,player:&mut Player)->Result<(),Error> {
        // 加载本地文件path
        let swf_movie = SwfMovie::from_path(url)?;
        // player.lock().unwrap().set_root_movie(swf_movie);
        player.set_root_movie(swf_movie);
        Ok(())
    }
}