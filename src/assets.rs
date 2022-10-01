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
    pub player_acceleration: Coord,
    pub initial_balloons: usize,
    pub balloon_mass: R32,
    pub player_mass: R32,
    pub balloon_drag: R32,
    pub player_drag: R32,
    pub balloon_length: Coord,
    pub obstacles: ObstacleConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, geng::Assets)]
#[asset(json)]
pub struct ObstacleConfig {
    pub spawn_area_width: Coord,
    pub min_speed: Coord,
    pub max_speed: Coord,
    pub min_height: Coord,
    pub max_gen_height: Coord,
    pub min_dh: Coord,
    pub max_dh: Coord,
}

#[derive(geng::Assets)]
pub struct Sprites {
    pub player: ugli::Texture,
    pub balloon: ugli::Texture,
}
