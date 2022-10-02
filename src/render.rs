use super::*;

use geng::Camera2d;
use model::*;

pub struct Render {
    geng: Geng,
    assets: Rc<Assets>,
    camera: Camera2d,
    camera_target: Vec2<f32>,
    backgrounds: Vec<usize>,
}

const CAMERA_INTERPOLATION: f32 = 0.5;
const FOV: f32 = 10.0;
const FOV_HORIZONTAL: f32 = FOV * 16.0 / 9.0;

const TEXT_COLOR: Rgba<f32> = Rgba::BLACK;

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
            backgrounds: Vec::new(),
        }
    }

    pub fn update(&mut self, model: &Model, delta_time: f32) {
        self.camera_target.y = model.player.position.y.as_f32() + 2.0;
        self.camera.center +=
            (self.camera_target - self.camera.center) / CAMERA_INTERPOLATION * delta_time;

        let target_height = self.camera.center.y + FOV;
        let mut current_height = FOV * self.backgrounds.len() as f32;
        let mut rng = global_rng();
        while current_height < target_height {
            let index = match self.backgrounds.last() {
                Some(last) => (0..self.assets.sprites.background.len())
                    .filter(|i| i != last)
                    .choose(&mut rng),
                None => (0..self.assets.sprites.background.len()).choose(&mut rng),
            }
            .expect("Failed to select a random background");
            self.backgrounds.push(index);
            current_height += FOV;
        }
    }

    pub fn draw(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        {
            // Background
            let mut height = 0.0;
            for &index in &self.backgrounds {
                let aabb = AABB::point(vec2(-FOV_HORIZONTAL / 2.0, height))
                    .extend_positive(vec2(FOV_HORIZONTAL, FOV));
                let quad = draw_2d::TexturedQuad::new(aabb, &self.assets.sprites.background[index]);
                geng::Draw2d::draw_2d(&quad, &self.geng, framebuffer, &self.camera);
                height += FOV;
            }

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
                ObstacleType::Plane => self
                    .assets
                    .sprites
                    .airplane
                    .get_frame(obstacle.animation_time),
                ObstacleType::Helicopter1 => self
                    .assets
                    .sprites
                    .helicopter
                    .get_frame(obstacle.animation_time),
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

        self.draw_ui(model, framebuffer);
    }

    fn draw_ui(&mut self, model: &Model, framebuffer: &mut ugli::Framebuffer) {
        let framebuffer_size = framebuffer.size().map(|x| x as f32);
        let screen = |anchor: Vec2<f32>, offset: Vec2<f32>| -> Vec2<f32> {
            framebuffer_size * anchor + offset
        };
        let font = &**self.geng.default_font();

        if model.player.alive && !model.player.balloons.is_empty() {
            // Score
            let text = format!("Score: {}", model.score);
            let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                .scale_uniform(20.0)
                .align_bounding_box(vec2(0.0, 1.0))
                .translate(screen(vec2(0.0, 1.0), vec2(20.0, -20.0)));
            geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);

            // High score
            let text = format!("High Score: {}", model.high_score);
            let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                .scale_uniform(20.0)
                .align_bounding_box(vec2(1.0, 1.0))
                .translate(screen(vec2(1.0, 1.0), vec2(-20.0, -20.0)));
            geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);
        } else {
            if !model.player.alive {
                // Death message
                let text = "You got hit :(";
                let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                    .scale_uniform(40.0)
                    .align_bounding_box(vec2(0.5, 0.5))
                    .translate(screen(vec2(0.5, 0.5), vec2(0.0, 250.0)));
                geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);
            } else {
                // Out of balloons message
                let text = "All your balloons popped :(";
                let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                    .scale_uniform(40.0)
                    .align_bounding_box(vec2(0.5, 0.5))
                    .translate(screen(vec2(0.5, 0.5), vec2(0.0, 250.0)));
                geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);
            }

            // Score
            let text = format!("You scored: {}", model.score);
            let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                .scale_uniform(40.0)
                .align_bounding_box(vec2(0.5, 0.5))
                .translate(screen(vec2(0.5, 0.5), vec2(0.0, 75.0)));
            geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);
            let text = format!("High score: {}", model.high_score);
            let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                .scale_uniform(40.0)
                .align_bounding_box(vec2(0.5, 0.5))
                .translate(screen(vec2(0.5, 0.5), vec2(0.0, -75.0)));
            geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);

            // Reset prompt
            let text = "Press R to Restart";
            let text = draw_2d::Text::unit(font, text, TEXT_COLOR)
                .scale_uniform(40.0)
                .align_bounding_box(vec2(0.5, 0.5))
                .translate(screen(vec2(0.5, 0.5), vec2(0.0, -250.0)));
            geng::Draw2d::draw_2d(&text, &self.geng, framebuffer, &geng::PixelPerfectCamera);
        }
    }
}
