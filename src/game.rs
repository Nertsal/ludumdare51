use super::*;

use model::*;
use render::Render;

pub struct Game {
    geng: Geng,
    render: Render,
    model: Model,
}

impl Game {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            render: Render::new(geng, assets),
            model: Model::new(assets.config.clone()),
        }
    }

    fn control(&mut self, delta_time: Time) {
        let geng = &self.geng;
        fn is_pressed(geng: &Geng, keys: impl IntoIterator<Item = geng::Key>) -> bool {
            let window = geng.window();
            keys.into_iter().any(|key| window.is_key_pressed(key))
        }
        use geng::Key;
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
        self.model.player.velocity +=
            direction.map(|x| r32(x as f32)) * self.model.config.player_acceleration * delta_time;
    }
}

impl geng::State for Game {
    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Rgba::from_rgb(0.0, 0.7, 0.7)), None, None);
        self.render.draw(&self.model, framebuffer);
    }

    fn handle_event(&mut self, _event: geng::Event) {}

    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.control(delta_time);
        self.model.update(delta_time);
        self.render.update(&self.model, delta_time.as_f32());
    }
}
