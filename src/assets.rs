use super::*;
use crate::model::*;

#[derive(geng::Assets)]
pub struct Assets {
    pub sprites: Sprites,
    pub config: Config,
}

#[derive(Debug, Clone, Serialize, Deserialize, geng::Assets)]
#[asset(json)]
pub struct Config {
    pub gravity: Vec2<Coord>,
    pub balloon_pop_time: Time,
    pub player_acceleration: Coord,
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
    pub min_delay: Time,
    pub max_delay: Time,
}

#[derive(geng::Assets)]
pub struct Sprites {
    pub player: ugli::Texture,
    pub balloon: ugli::Texture,
    pub airplane: ugli::Texture,
    pub helicopter: ugli::Texture,
    pub helicopter2: ugli::Texture,
    #[asset(path = "clouds/*.png", range = "0..3")]
    pub clouds: Vec<ugli::Texture>,
}
