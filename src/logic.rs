use super::*;

use model::*;

pub struct Logic<'a> {
    delta_time: Time,
    model: &'a mut Model,
}

impl Model {
    pub fn update(&mut self, delta_time: Time) {
        let mut logic = Logic {
            delta_time,
            model: self,
        };
        logic.process();
    }
}

impl Logic<'_> {
    pub fn process(&mut self) {
        self.apply_gravity();
        self.player_balloon();
        self.collisions();
        self.movement();
        self.generation();
    }

    fn apply_gravity(&mut self) {
        self.model.player.velocity += self.model.config.gravity * self.delta_time;
        for balloon in &mut self.model.balloons {
            balloon.velocity -= self.model.config.gravity * self.delta_time;
        }
    }

    fn movement(&mut self) {
        {
            let player = &mut self.model.player;
            player.velocity -= player.velocity * player.drag * self.delta_time;
            player.position += player.velocity * self.delta_time;
        }
        for balloon in &mut self.model.balloons {
            balloon.velocity -= balloon.velocity * balloon.drag * self.delta_time;
            balloon.position += balloon.velocity * self.delta_time;
            if balloon.attached_to_player {
                let delta = balloon.position - self.model.player.position;
                balloon.position = self.model.player.position + delta.clamp_len(..=balloon.length);
            }
        }
        for obstacle in &mut self.model.obstacles {
            obstacle.position += obstacle.velocity * self.delta_time;
        }
    }

    fn collisions(&mut self) {
        {
            // Player-ground
            let player = &mut self.model.player;
            if player.position.y < Coord::ZERO {
                player.position.y = Coord::ZERO;
                player.velocity = Vec2::ZERO;
            }
        }

        // Balloon-balloon
        let ids: Vec<Id> = self.model.balloons.ids().copied().collect();
        for id in ids {
            let mut balloon = self.model.balloons.remove(&id).unwrap();
            for other in &mut self.model.balloons {
                collide(
                    &mut balloon.position,
                    &mut balloon.velocity,
                    balloon.radius,
                    balloon.mass,
                    &mut other.position,
                    &mut other.velocity,
                    other.radius,
                    other.mass,
                );
            }
            self.model.balloons.insert(balloon);
        }

        // Balloon-obstacle
        for obstacle in &mut self.model.obstacles {
            for balloon in &mut self.model.balloons {
                let delta = obstacle.position - balloon.position;
                let penetration = obstacle.radius + balloon.radius - delta.len();
                if penetration > Coord::ZERO {
                    // Pop the balloon
                    balloon.popped = true;
                }
            }
        }
        self.model.balloons.retain(|b| !b.popped);
    }

    fn player_balloon(&mut self) {
        let player = &mut self.model.player;
        let mut alive_balloons = Vec::new();
        for balloon in &player.balloons {
            let balloon = match self.model.balloons.get_mut(balloon) {
                None => continue,
                Some(b) => {
                    alive_balloons.push(*balloon);
                    b
                }
            };
            let delta = balloon.position - player.position;
            if delta.len() < balloon.length {
                // No tension
                continue;
            }
            let (p_vel, b_vel) = collide_impulses(
                player.mass,
                player.velocity,
                balloon.mass,
                balloon.velocity,
                delta,
                r32(0.0),
            );
            player.velocity = p_vel;
            balloon.velocity = b_vel;
        }
        player.balloons = alive_balloons;
    }

    fn generation(&mut self) {
        let mut rng = global_rng();
        let config = &self.model.config.obstacles;
        let gen_height = self.model.player.position.y + config.max_gen_height;
        while self.model.last_gen_height <= gen_height {
            let height = self.model.last_gen_height + rng.gen_range(config.min_dh..=config.max_dh);
            self.model.last_gen_height = height;
            let side = r32((rng.gen_range(0..=1) * 2 - 1) as f32);
            let radius = r32(0.3);
            let speed = rng.gen_range(config.min_speed..=config.max_speed);
            let x = (config.spawn_area_width + radius) * side;
            let obstacle_type = *vec![
                ObstacleType::Plane,
                ObstacleType::Helicopter1,
                ObstacleType::Helicopter2,
            ]
            .choose(&mut rng)
            .expect("Failed to select the obstacle type");
            let obstacle = Obstacle {
                id: self.model.id_gen.gen(),
                obstacle_type,
                position: vec2(x, height),
                velocity: vec2(-side * speed, Coord::ZERO),
                radius,
            };
            self.model.obstacles.insert(obstacle);
        }
    }
}

fn collide(
    position_a: &mut Vec2<Coord>,
    velocity_a: &mut Vec2<Coord>,
    radius_a: Coord,
    mass_a: R32,
    position_b: &mut Vec2<Coord>,
    velocity_b: &mut Vec2<Coord>,
    radius_b: Coord,
    mass_b: R32,
) {
    let delta = *position_b - *position_a;
    let penetration = radius_a + radius_b - delta.len();
    if penetration > Coord::ZERO {
        let direction = delta.normalize_or_zero();
        let penetration = penetration * r32(0.5);
        let delta = direction * penetration;
        *position_a -= delta;
        *position_b += delta;
        let delta = *position_b - *position_a;
        let (a_vel, b_vel) =
            collide_impulses(mass_a, *velocity_a, mass_b, *velocity_b, delta, r32(0.0));
        *velocity_a = a_vel;
        *velocity_b = b_vel;
    }
}

fn collide_impulses(
    mass_a: R32,
    velocity_a: Vec2<Coord>,
    mass_b: R32,
    velocity_b: Vec2<Coord>,
    delta: Vec2<Coord>,
    elasticity: Coord,
) -> (Vec2<Coord>, Vec2<Coord>) {
    let relative_impulse = velocity_b * mass_b - velocity_a * mass_a;
    let direction = delta.normalize_or_zero();
    let relative_proj =
        direction * Vec2::dot(direction, relative_impulse) * (elasticity / r32(2.0) + r32(0.5));
    (
        velocity_a + relative_proj / mass_a,
        velocity_b - relative_proj / mass_b,
    )
}
