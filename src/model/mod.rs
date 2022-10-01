use super::*;

mod id;

pub use id::*;

pub type Time = R32;
pub type Coord = R32;

pub struct Model {
    pub config: Config,
    pub id_gen: IdGenerator,
    pub player: Player,
    pub balloons: Collection<Balloon>,
}

pub struct Player {
    pub mass: R32,
    pub position: Vec2<Coord>,
    pub velocity: Vec2<Coord>,
    pub radius: Coord,
    pub drag: R32,
    pub balloons: Vec<Id>,
}

#[derive(HasId)]
pub struct Balloon {
    pub id: Id,
    pub mass: R32,
    pub position: Vec2<Coord>,
    pub velocity: Vec2<Coord>,
    pub radius: Coord,
    pub length: Coord,
    pub drag: R32,
    pub color: Rgba<f32>,
    pub attached_to_player: bool,
}

impl Model {
    pub fn new(config: Config) -> Self {
        let mut id_gen = IdGenerator::new();
        let mut rng = global_rng();

        let mut balloons = Collection::new();
        for _ in 0..config.initial_balloons {
            let x = rng.gen_range(-1.0..=1.0);
            let y = rng.gen_range(-0.1..=0.1);
            let balloon = Balloon {
                id: id_gen.gen(),
                mass: config.balloon_mass,
                position: vec2(0.0 + x, 2.0 + y).map(r32),
                velocity: Vec2::ZERO,
                radius: r32(0.25),
                length: r32(1.0),
                drag: config.balloon_drag,
                color: Rgba::RED,
                attached_to_player: true,
            };
            balloons.insert(balloon);
        }

        Self {
            id_gen,
            player: Player {
                mass: config.player_mass,
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                radius: r32(0.5),
                drag: config.player_drag,
                balloons: balloons.ids().copied().collect(),
            },
            balloons,
            config,
        }
    }
}
