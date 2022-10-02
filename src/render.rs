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
const FOV: f32 = 10.0;
const FOV_HORIZONTAL: f32 = FOV * 16.0 / 9.0;

impl Render {
    pub fn new(geng: &Geng, assets: &Rc<Assets>) -> Self {
        Self {
            geng: geng.clone(),
            assets: assets.clone(),
            camera: Camera2d {
                center: vec2(0.0, 2.0),
                fov: 10.0,
                rotation: 0.0,
            },
            camera_target: Vec2::ZERO,
        }
    }

    pub fn update(&mut self, model: &Model, delta_time: f32) {
        self.camera_target.y = model.player.position.y.as_f32() + 2.0;
        self.camera.center +=
            (self.camera_target - self.camera.center) / CAMERA_INTERPOLATION * delta_time;
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        {
            // Start area
            let aabb = AABB::point(vec2(0.0, -3.0))
                .extend_symmetric(vec2(FOV_HORIZONTAL, 0.0) / 2.0)
                .extend_up(FOV);
            let quad = draw_2d::TexturedQuad::new(aabb, &self.assets.sprites.start[2]);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
            let quad = draw_2d::TexturedQuad::new(aabb, &self.assets.sprites.start[1]);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
            let quad = draw_2d::TexturedQuad::new(aabb, &self.assets.sprites.start[0]);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }

        // Clouds
        for cloud in &model.clouds {
            let aabb = AABB::point(cloud.position)
                .extend_uniform(cloud.radius * r32(1.5))
                .map(|x| x.as_f32());
            let texture = match cloud.cloud_type {
                CloudType::Cloud0 => &self.assets.sprites.clouds[0],
                CloudType::Cloud1 => &self.assets.sprites.clouds[1],
                CloudType::Cloud2 => &self.assets.sprites.clouds[2],
            };
            let quad = draw_2d::TexturedQuad::new(aabb, texture);
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
                    self.assets
                        .sprites
                        .helicopter2
                        .get_frame(obstacle.animation_time)
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

            if balloon.attached_to_player {
                let segment = Segment::new(
                    vec2(aabb.center().x, aabb.y_min + balloon.radius.as_f32() * 0.1),
                    model.player.position.map(|x| x.as_f32()),
                );
                let segment = draw_2d::Segment::new(segment, 0.02, Rgba::BLACK);
                geng::Draw2d::draw_2d(&segment, &self.geng, framebuffer, &self.camera);
            }

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
            let texture = self.assets.sprites.player.get_frame(player.animation_time);
            let quad = draw_2d::TexturedQuad::new(aabb, texture);
            geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
        }
    }
}
