use geng::prelude::*;

mod assets;
mod game;
mod logic;
mod model;
mod render;

use assets::*;

fn main() {
    logger::init().unwrap();
    geng::setup_panic_handler();

    let geng = Geng::new_with(geng::ContextOptions {
        title: "Ludum Dare 51 - Every 10 seconds".to_owned(),
        ..Default::default()
    });
    let assets = <Assets as geng::LoadAsset>::load(&geng, &static_path());

    geng::run(
        &geng,
        geng::LoadingScreen::new(&geng, geng::EmptyLoadingScreen, assets, {
            let geng = geng.clone();
            move |assets| {
                let mut assets = assets.expect("Failed to load assets");
                assets.process();
                let assets = Rc::new(assets);
                game::Game::new(&geng, &assets)
            }
        }),
    )
}
