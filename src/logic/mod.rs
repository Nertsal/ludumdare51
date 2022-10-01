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

        let player = &mut self.model.player;
        for balloon in &player.balloons {
            let balloon = self
                .model
                .balloons
                .get_mut(balloon)
                .expect("Failed to find the balloon");
            let delta = balloon.position - player.position;
            let direction = delta.normalize_or_zero();
            if delta.len() < balloon.length {
                // No tension
                continue;
            }
            let b_impulse = balloon.velocity * balloon.mass;
            let p_impulse = player.velocity * player.mass;
            let relative = b_impulse - p_impulse;
            let relative_proj = direction * Vec2::dot(direction, relative) * r32(0.5);
            balloon.velocity -= relative_proj / balloon.mass;
            player.velocity += relative_proj / player.mass;
        }

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
    }
}
