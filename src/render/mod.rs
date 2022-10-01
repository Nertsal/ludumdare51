use super::*;

use geng::Camera2d;
use model::*;

pub struct Render {
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera2d,
}

impl Render {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera2d {
                center: Vec2::ZERO,
                fov: 10.0,
                rotation: 0.0,
            },
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        {
            // Ground
            let aabb = AABB::ZERO
                .extend_symmetric(vec2(100.0, 0.0))
                .extend_down(100.0);
            let quad = draw_2d::Quad::new(aabb, Rgba::from_rgb(0.3, 0.6, 0.0));
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }

        {
            // Player
            let player = &model.player;
            let aabb = AABB::point(player.position)
                .extend_uniform(player.radius)
                .map(|x| x.as_f32());
            let quad = draw_2d::Quad::new(aabb, Rgba::WHITE);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }

        // Balloons
        for balloon in &model.balloons {
            let aabb = AABB::point(balloon.position)
                .extend_uniform(balloon.radius)
                .map(|x| x.as_f32());
            let quad = draw_2d::Quad::new(aabb, balloon.color);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }
    }
}
