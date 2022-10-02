use super::*;

use geng::Camera2d;
use model::*;

pub struct Render {
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera2d,
    camera_target: Vec2<f32>,
}

const CAMERA_INTERPOLATION: f32 = 0.5;

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
            camera_target: Vec2::ZERO,
        }
    }

    pub fn update(&mut self, model: &Model, delta_time: f32) {
        self.camera_target.y = model.player.position.y.as_f32() + 3.0;
        self.camera.center +=
            (self.camera_target - self.camera.center) / CAMERA_INTERPOLATION * delta_time;
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

        // Obstacles
        for obstacle in &model.obstacles {
            let mut aabb = AABB::point(obstacle.position)
                .extend_uniform(obstacle.radius * r32(1.5))
                .map(|x| x.as_f32());
            let mut mirror = obstacle.velocity.x < Coord::ZERO;
            let texture = match obstacle.obstacle_type {
                ObstacleType::Plane => &self.assets.sprites.airplane,
                ObstacleType::Helicopter1 => &self.assets.sprites.helicopter,
                ObstacleType::Helicopter2 => {
                    mirror = !mirror;
                    &self.assets.sprites.helicopter2
                }
            };
            if mirror {
                std::mem::swap(&mut aabb.x_min, &mut aabb.x_max);
            }
            let quad = draw_2d::TexturedQuad::new(aabb, texture);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }

        // Balloons
        for balloon in &model.balloons {
            let aabb = AABB::point(balloon.position)
                .extend_uniform(balloon.radius * r32(1.5))
                .map(|x| x.as_f32());

            let segment = Segment::new(
                vec2(aabb.center().x, aabb.y_min + balloon.radius.as_f32() * 0.1),
                model.player.position.map(|x| x.as_f32()),
            );
            let segment = draw_2d::Segment::new(segment, 0.02, Rgba::BLACK);
            geng::Draw2d::draw_2d(&segment, &self.geng, framebuffer, &self.camera);

            let quad =
                draw_2d::TexturedQuad::colored(aabb, &self.assets.sprites.balloon, balloon.color);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }

        {
            // Player
            let player = &model.player;
            let aabb = AABB::point(player.position)
                .extend_uniform(player.radius)
                .map(|x| x.as_f32());
            let quad = draw_2d::TexturedQuad::new(aabb, &self.assets.sprites.player);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }
    }
}
