use super::*;

use model::*;
use render::Render;

pub struct Game {
    geng: Geng,
    assets: Rc<Assets>,
    render: Render,
    model: Model,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            render: Render::new(geng, assets),
            model: Model::new(assets.config.clone(), assets),
        }
    }

    fn control(&mut self, delta_time: Time) {
        let geng = &self.geng;
        fn is_pressed(geng: &Geng, keys: impl IntoIterator<Item = geng::Key>) -> bool {
            let window = geng.window();
            keys.into_iter().any(|key| window.is_key_pressed(key))
        }
        use geng::Key;

        if is_pressed(geng, [Key::R]) {
            self.reset();
            return;
        }

        if !self.model.player.alive {
            self.model.player_control_velocity = Vec2::ZERO;
            return;
        }
        let mut direction: Vec2<i32> = Vec2::ZERO;
        if is_pressed(geng, [Key::A, Key::Left]) {
            direction.x -= 1;
        }
        if is_pressed(geng, [Key::D, Key::Right]) {
            direction.x += 1;
        }
        if is_pressed(geng, [Key::S, Key::Down]) {
            direction.y -= 1;
        }
        if is_pressed(geng, [Key::W, Key::Up]) {
            direction.y += 1;
        }
        let config = &self.model.config;
        let speed_y = if self.model.player.balloons.is_empty() {
            Coord::ZERO
        } else if direction.y < 0 {
            config.player_speed_v_down
        } else {
            config.player_speed_v_up
        };
        let speed = vec2(config.player_speed_h, speed_y);
        let target_speed = direction.map(|x| r32(x as f32)) * speed;
        let acc = r32(10.0);
        self.model.player_control_velocity += (target_speed - self.model.player_control_velocity)
            .clamp_len(Coord::ZERO..=acc * delta_time);
    }

    fn reset(&mut self) {
        self.model.reset();
        self.render = Render::new(&self.geng, &self.assets);
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::from_rgb(0.0, 0.7, 0.7)), None, None);
        self.render.draw(&self.model, framebuffer);
    }

    fn handle_event(&mut self, event: geng::Event) {
        if let geng::Event::KeyDown { key: geng::Key::R } = event {
            self.reset();
        }
    }

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.control(delta_time);
        self.model.update(delta_time);
        self.render.update(&self.model, delta_time.as_f32());
    }
}
