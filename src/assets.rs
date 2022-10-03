use super::*;
use crate::model::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub sprites: Sprites,
    pub config: Config,
    pub sounds: Sounds,
}

#[derive(Deref)]
pub struct Animation {
    #[deref]
    pub frames: Vec<ugli::Texture>,
}

#[derive(Debug, Clone, Serialize, Deserialize, geng::Assets)]
#[asset(json)]
pub struct Config {
    pub gravity: Vec2<Coord>,
    pub arena_width: Coord,
    pub balloon_pop_time: Time,
    pub player_speed_h: Coord,
    pub player_speed_v_down: Coord,
    pub player_speed_v_up: Coord,
    pub initial_balloons: usize,
    pub balloon_mass: R32,
    pub player_mass: R32,
    pub balloon_drag: R32,
    pub balloon_attached_drag: R32,
    pub player_drag: R32,
    pub balloon_length: Coord,
    pub obstacles: ObstacleConfig,
    pub clouds: ObstacleConfig,
    pub balloons: BalloonsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, geng::Assets)]
#[asset(json)]
pub struct BalloonsConfig {
    pub spawn_area_width: Coord,
    pub min_height: Coord,
    pub below_player: Coord,
    pub min_delay: Time,
    pub max_delay: Time,
}

#[derive(Debug, Clone, Serialize, Deserialize, geng::Assets)]
#[asset(json)]
pub struct ObstacleConfig {
    pub spawn_area_width: Coord,
    pub min_speed: Coord,
    pub max_speed: Coord,
    pub min_height: Coord,
    pub above_player: Coord,
    pub below_player: Coord,
    pub ahead_of_player: Time,
    pub min_delay: Time,
    pub max_delay: Time,
}

#[derive(geng::Assets)]
pub struct Sprites {
    pub player: Animation,
    pub player_dead: Animation,
    pub balloon: ugli::Texture,
    pub airplane: Animation,
    pub helicopter: Animation,
    pub helicopter2: Animation,
    #[asset(path = "clouds/*.png", range = "0..3")]
    pub clouds: Vec<ugli::Texture>,
    #[asset(path = "start/*.png", range = "0..3")]
    pub start: Vec<ugli::Texture>,
    #[asset(path = "background/*.png", range = "0..3")]
    pub background: Vec<ugli::Texture>,
}

#[derive(geng::Assets)]
pub struct Sounds {
    #[asset(path = "nya/*.wav", range = "0..2")]
    pub nya: Vec<geng::Sound>,
    pub hit: geng::Sound,
    pub splash: geng::Sound,
    pub pop: geng::Sound,
    pub helicopter: geng::Sound,
    pub wind: geng::Sound,
}

impl Assets {
    pub fn process(&mut self) {
        self.sounds.helicopter.looped = true;
        self.sounds.wind.looped = true;
    }
}

impl Animation {
    pub fn get_frame(&self, time: Time) -> &ugli::Texture {
        let i = (time.as_f32() * self.frames.len() as f32).floor() as usize;
        &self.frames[i]
    }
}

impl geng::LoadAsset for Animation {
    fn load(geng: &Geng, path: &std::path::Path) -> geng::AssetFuture<Self> {
        let data = <Vec<u8> as geng::LoadAsset>::load(geng, path);
        let geng = geng.clone();
        async move {
            let data = data.await?;
            use image::AnimationDecoder;
            Ok(Self {
                frames: image::codecs::png::PngDecoder::new(data.as_slice())
                    .unwrap()
                    .apng()
                    .into_frames()
                    .map(|frame| {
                        let frame = frame.unwrap();
                        ugli::Texture::from_image_image(geng.ugli(), frame.into_buffer())
                    })
                    .collect(),
            })
        }
        .boxed_local()
    }
    const DEFAULT_EXT: Option<&'static str> = Some("png");
}
