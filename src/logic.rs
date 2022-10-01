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
            player.velocity -= player.velocity.sqr() * player.drag * self.delta_time;
            player.position += player.velocity * self.delta_time;
        }
        for balloon in &mut self.model.balloons {
            balloon.velocity -= balloon.velocity.sqr() * balloon.drag * self.delta_time;
            balloon.position += balloon.velocity * self.delta_time;
            if balloon.attached_to_player {
                let delta = balloon.position - self.model.player.position;
                balloon.position = self.model.player.position + delta.clamp_len(..=balloon.length);
            }
        }
    }

    fn collisions(&mut self) {
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
    }

    fn player_balloon(&mut self) {
        let player = &mut self.model.player;
        for balloon in &player.balloons {
            let balloon = self
                .model
                .balloons
                .get_mut(balloon)
                .expect("Failed to find the balloon");
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
    }
    let delta = *position_b - *position_a;
    let (a_vel, b_vel) =
        collide_impulses(mass_a, *velocity_a, mass_b, *velocity_b, delta, r32(0.0));
    *velocity_a = a_vel;
    *velocity_b = b_vel;
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
