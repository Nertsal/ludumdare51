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
    pub initial_balloons: usize,
    pub balloon_mass: R32,
    pub player_mass: R32,
    pub balloon_drag: R32,
    pub player_drag: R32,
}

#[derive(geng::Assets)]
pub struct Sprites {
    pub player: ugli::Texture,
}
