use super::*;

mod id;

pub use id::*;

pub type Time = R32;
pub type Coord = R32;
pub type Score = u64;

const HIGH_SCORE_SAVE_FILE: &str = "caterpillar_save";

pub struct Model {
    pub config: Config,
    pub assets: Rc<Assets>,
    pub id_gen: IdGenerator,
    pub next_obstacle: Time,
    pub next_cloud: Time,
    pub next_balloon: Time,
    pub next_pop: Time,
    pub player: Player,
    pub balloons: Collection<Balloon>,
    pub obstacles: Collection<Obstacle>,
    pub clouds: Collection<Cloud>,
    pub high_score: Score,
    pub score: Score,
}

pub struct Player {
    pub animation_time: Time,
    pub alive: bool,
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
    pub popped: bool,
}

#[derive(HasId)]
pub struct Obstacle {
    pub id: Id,
    pub animation_speed: R32,
    pub animation_time: Time,
    pub obstacle_type: ObstacleType,
    pub position: Vec2<Coord>,
    pub velocity: Vec2<Coord>,
    pub radius: Coord,
}

#[derive(Debug, Clone, Copy)]
pub enum ObstacleType {
    Plane,
    Helicopter1,
    Helicopter2,
}

#[derive(HasId)]
pub struct Cloud {
    pub id: Id,
    pub cloud_type: CloudType,
    pub position: Vec2<Coord>,
    pub velocity: Vec2<Coord>,
    pub radius: Coord,
}

#[derive(Debug, Clone, Copy)]
pub enum CloudType {
    Cloud0,
    Cloud1,
    Cloud2,
}

impl Model {
    pub fn new(config: Config, assets: &Rc<Assets>) -> Self {
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
                length: config.balloon_length,
                drag: config.balloon_drag,
                color: Rgba::RED,
                attached_to_player: true,
                popped: false,
            };
            balloons.insert(balloon);
        }

        Self {
            id_gen,
            assets: assets.clone(),
            next_obstacle: Time::ZERO,
            next_cloud: Time::ZERO,
            next_balloon: Time::ZERO,
            next_pop: config.balloon_pop_time,
            player: Player {
                animation_time: Time::ZERO,
                alive: true,
                mass: config.player_mass,
                position: Vec2::ZERO,
                velocity: Vec2::ZERO,
                radius: r32(0.3),
                drag: config.player_drag,
                balloons: balloons.ids().copied().collect(),
            },
            balloons,
            obstacles: default(),
            clouds: default(),
            config,
            high_score: batbox::preferences::load(HIGH_SCORE_SAVE_FILE).unwrap_or(Score::ZERO),
            score: Score::ZERO,
        }
    }

    pub fn reset(&mut self) {
        self.high_score = self.high_score.max(self.score);
        batbox::preferences::save(HIGH_SCORE_SAVE_FILE, &self.high_score);
        *self = Model::new(self.config.clone(), &self.assets);
    }
}
