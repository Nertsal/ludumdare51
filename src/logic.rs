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

    pub fn sound_volume(&self, position: Vec2<Coord>) -> f64 {
        let distance = (position - self.player.position).len().as_f32();
        (1.0 - (distance / 10.0).sqr()).max(0.0) as f64 * self.volume
    }

    pub fn play_sound(&self, sound: &geng::Sound, position: Vec2<Coord>) {
        let mut effect = sound.effect();
        let volume = self.sound_volume(position);
        effect.set_volume(dbg!(volume));
        effect.play();
    }
}

impl Logic<'_> {
    pub fn process(&mut self) {
        self.sounds();
        self.update_score();
        self.apply_gravity();
        self.player_balloon();
        self.pop();
        self.collisions();
        self.movement();
        self.generation();
        self.animations();
    }

    fn sounds(&mut self) {
        // Wind
        let volume = (self.model.player.position.y.max(Coord::ZERO) / r32(20.0))
            .sqrt()
            .as_f32()
            .clamp(0.0, 1.0) as f64
            * self.model.volume;
        self.model.wind_sound.set_volume(volume);

        // Helicopter
        let volume = self
            .model
            .obstacles
            .iter()
            .filter(|obstacle| {
                matches!(
                    obstacle.obstacle_type,
                    ObstacleType::Helicopter1 | ObstacleType::Helicopter2
                )
            })
            .map(|helicopter| r64(self.model.sound_volume(helicopter.position)))
            .max()
            .unwrap_or(R64::ZERO)
            .as_f32()
            .into();
        self.model.helicopter_sound.set_volume(volume);
    }

    fn update_score(&mut self) {
        let score = self
            .model
            .player
            .position
            .y
            .floor()
            .max(Coord::ZERO)
            .as_f32() as Score;
        self.model.score = self.model.score.max(score);
    }

    fn pop(&mut self) {
        self.model.next_pop -= self.delta_time;
        if self.model.next_pop < Time::ZERO {
            // Pop a balloon
            let mut rng = global_rng();
            if let Some(i) = (0..self.model.player.balloons.len()).choose(&mut rng) {
                let balloon = self.model.player.balloons.remove(i);
                let balloon = self
                    .model
                    .balloons
                    .remove(&balloon)
                    .expect("Popped an non-existing balloon");
                self.model
                    .play_sound(&self.model.assets.sounds.pop, balloon.position);
            }
            self.model.next_pop = self.model.config.balloon_pop_time;
        }
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
            player.velocity *=
                Coord::ONE - player.velocity.len_sqr() * player.drag * self.delta_time;
            player.position +=
                (player.velocity + self.model.player_control_velocity) * self.delta_time;
            player.position.x = player.position.x.clamp_abs(self.model.config.arena_width);
        }
        for balloon in &mut self.model.balloons {
            balloon.drag = if balloon.attached_to_player {
                self.model.config.balloon_attached_drag
            } else {
                self.model.config.balloon_drag
            };
            balloon.velocity *=
                Coord::ONE - balloon.velocity.len_sqr() * balloon.drag * self.delta_time;
            balloon.position += balloon.velocity * self.delta_time;
            if balloon.attached_to_player {
                let delta = balloon.position - self.model.player.position;
                balloon.position = self.model.player.position + delta.clamp_len(..=balloon.length);
            }
        }
        for obstacle in &mut self.model.obstacles {
            obstacle.position += obstacle.velocity * self.delta_time;
        }
        for cloud in &mut self.model.clouds {
            cloud.position += cloud.velocity * self.delta_time;
        }
    }

    fn animations(&mut self) {
        let update = |time: &mut Time, speed: R32| {
            *time += speed * self.delta_time;
            if *time >= Time::ONE {
                *time -= Time::ONE;
            }
        };

        update(&mut self.model.player.animation_time, r32(1.0));
        for obstacle in &mut self.model.obstacles {
            update(&mut obstacle.animation_time, obstacle.animation_speed);
        }
    }

    fn collisions(&mut self) {
        // Player-ground
        let player = &mut self.model.player;
        if player.position.y < Coord::ZERO {
            if player.velocity.y.abs() > self.model.config.gravity.y.abs() * r32(0.2) {
                self.model.assets.sounds.splash.play();
            }
            player.position.y = Coord::ZERO;
            player.velocity = Vec2::ZERO;
        }

        if player.alive {
            // Player-obstacles
            let mut kill = false;
            for obstacle in &self.model.obstacles {
                let delta = obstacle.position - player.position;
                let penetration = obstacle.radius + player.radius - delta.len();
                if penetration > Coord::ZERO {
                    // Kill the player
                    kill = true;
                    player.velocity += obstacle.velocity;
                    self.model.assets.sounds.hit.play();
                    break;
                }
            }
            if kill {
                self.kill_player();
            } else {
                // Player-balloon
                for balloon in &mut self.model.balloons {
                    if balloon.attached_to_player {
                        continue;
                    }
                    let delta = balloon.position - player.position;
                    let penetration = balloon.radius + player.radius - delta.len();
                    if penetration > Coord::ZERO {
                        player.balloons.push(balloon.id);
                        balloon.attached_to_player = true;
                        if let Some(nya) = self.model.assets.sounds.nya.choose(&mut global_rng()) {
                            nya.play();
                        }
                    }
                }
            }
        }

        // Balloon-balloon
        let ids: Vec<Id> = self.model.balloons.ids().copied().collect();
        for id in ids {
            let mut balloon = self.model.balloons.remove(&id).unwrap();
            for other in &mut self.model.balloons {
                let collision = collide(
                    &mut balloon.position,
                    &mut balloon.velocity,
                    balloon.radius,
                    balloon.mass,
                    &mut other.position,
                    &mut other.velocity,
                    other.radius,
                    other.mass,
                );
                if collision && (balloon.attached_to_player ^ other.attached_to_player) {
                    if !balloon.attached_to_player {
                        balloon.attached_to_player = true;
                        self.model.player.balloons.push(balloon.id);
                    } else {
                        other.attached_to_player = true;
                        self.model.player.balloons.push(other.id);
                    }
                    if let Some(nya) = self.model.assets.sounds.nya.choose(&mut global_rng()) {
                        nya.play();
                    }
                }
            }
            self.model.balloons.insert(balloon);
        }

        // Balloon-obstacle
        let mut pops = Vec::new();
        for obstacle in &self.model.obstacles {
            for balloon in &mut self.model.balloons {
                if balloon.popped {
                    continue;
                }
                let delta = obstacle.position - balloon.position;
                let penetration = obstacle.radius + balloon.radius - delta.len();
                if penetration > Coord::ZERO {
                    // Pop the balloon
                    balloon.popped = true;
                    pops.push(balloon.position);
                }
            }
        }
        self.model.balloons.retain(|b| !b.popped);
        for pop in pops {
            self.model.play_sound(&self.model.assets.sounds.pop, pop);
        }
    }

    fn kill_player(&mut self) {
        let player = &mut self.model.player;
        player.alive = false;
        for balloon in player.balloons.drain(..) {
            if let Some(balloon) = self.model.balloons.get_mut(&balloon) {
                balloon.attached_to_player = false;
            }
        }
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

        // Obstacles
        let config = &self.model.config.obstacles;
        self.model.next_obstacle -= self.delta_time;
        if self.model.next_obstacle < Time::ZERO {
            let ahead = self.model.player.velocity.y * config.ahead_of_player;
            let height = self.model.player.position.y
                + ahead
                + rng.gen_range(-config.below_player..=config.above_player);
            if height > config.min_height {
                let side = r32((rng.gen_range(0..=1) * 2 - 1) as f32);
                let radius = r32(0.5);
                let speed = rng.gen_range(config.min_speed..=config.max_speed);
                let x = (config.spawn_area_width + radius) * side;
                let (obstacle_type, animation_speed) = *vec![
                    (ObstacleType::Plane, r32(1.0)),
                    (ObstacleType::Helicopter1, r32(5.0)),
                    (ObstacleType::Helicopter2, r32(5.0)),
                ]
                .choose(&mut rng)
                .expect("Failed to select the cloud type");
                let obstacle = Obstacle {
                    id: self.model.id_gen.gen(),
                    animation_speed,
                    animation_time: Time::ZERO,
                    obstacle_type,
                    position: vec2(x, height),
                    velocity: vec2(-side * speed, Coord::ZERO),
                    radius,
                };
                self.model.obstacles.insert(obstacle);
            }

            let delay = rng.gen_range(config.min_delay..=config.max_delay);
            self.model.next_obstacle += delay;
        }

        // Clouds
        let config = &self.model.config.clouds;
        self.model.next_obstacle -= self.delta_time;
        if self.model.next_obstacle < Time::ZERO {
            let ahead = self.model.player.velocity.y * config.ahead_of_player;
            let height = self.model.player.position.y
                + ahead
                + rng.gen_range(-config.below_player..=config.above_player);
            if height > config.min_height {
                let side = r32((rng.gen_range(0..=1) * 2 - 1) as f32);
                let radius = r32(0.5);
                let speed = rng.gen_range(config.min_speed..=config.max_speed);
                let x = (config.spawn_area_width + radius) * side;
                let cloud_type = *vec![CloudType::Cloud0, CloudType::Cloud1, CloudType::Cloud2]
                    .choose(&mut rng)
                    .expect("Failed to select the obstacle type");
                let cloud = Cloud {
                    id: self.model.id_gen.gen(),
                    cloud_type,
                    position: vec2(x, height),
                    velocity: vec2(-side * speed, Coord::ZERO),
                    radius,
                };
                self.model.clouds.insert(cloud);
            }

            let delay = rng.gen_range(config.min_delay..=config.max_delay);
            self.model.next_obstacle += delay;
        }

        // Balloons
        self.model.next_balloon -= self.delta_time;
        if self.model.next_balloon < Time::ZERO {
            let config = &self.model.config.balloons;
            let y = self.model.player.position.y - config.below_player;
            if y > config.min_height {
                let x = r32(rng.gen_range(-1.0..=1.0)) * config.spawn_area_width;
                let color = *self
                    .model
                    .config
                    .balloon_colors
                    .choose(&mut rng)
                    .expect("Failed to select balloon color");
                let balloon = Balloon {
                    id: self.model.id_gen.gen(),
                    mass: self.model.config.balloon_mass,
                    position: vec2(x, y),
                    velocity: Vec2::ZERO,
                    radius: r32(0.25),
                    length: self.model.config.balloon_length,
                    drag: self.model.config.balloon_drag,
                    color,
                    attached_to_player: false,
                    popped: false,
                };
                self.model.balloons.insert(balloon);
                let delay = rng.gen_range(config.min_delay..=config.max_delay);
                self.model.next_balloon += delay;
            }
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
) -> bool {
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
        true
    } else {
        false
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
