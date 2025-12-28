use std::{cell::RefCell, env, rc::Rc};

use minesweeper_slint::mine_sweeper_ui::{
    GameConfig, MainWindow, expand_selection, fill_grid, model_grid_to_vec2d, new_grid,
    vec2d_to_model_grid,
};
use slint::{ComponentHandle, VecModel};

fn main() -> Result<(), slint::PlatformError> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }
    let main_window = MainWindow::new()?;

    // Global Configs
    let game_config = Rc::new(RefCell::new(GameConfig::default()));

    // Empty Grid
    let empty_grid = new_grid(&*game_config.borrow());
    let empty_grid_model = VecModel::from_slice(&empty_grid);
    main_window.set_buttons_grid(empty_grid_model);

    // First Move Occured
    let main_window_weak = main_window.as_weak();
    let game_config_cloned = game_config.clone();
    main_window.on_first_move_occured(move |position| {
        let tiles = fill_grid(&*game_config_cloned.borrow(), position);
        let model = vec2d_to_model_grid(tiles);
        main_window_weak.unwrap().set_buttons_grid(model);
    });

    // Quit Button
    let main_window_weak = main_window.as_weak();
    main_window.on_close(move || {
        main_window_weak.unwrap().hide().unwrap();
    });

    // Restart Button
    let main_window_weak = main_window.as_weak();
    let game_config_cloned = game_config.clone();
    main_window.on_restart(move || {
        let empty_grid = new_grid(&*game_config_cloned.borrow());
        let empty_grid_model = VecModel::from_slice(&empty_grid);
        main_window_weak.unwrap().set_buttons_grid(empty_grid_model);
    });

    // Expand Selection
    let main_window_weak = main_window.as_weak();
    let game_config_cloned = game_config.clone();
    main_window.on_expand_selection(move |position| {
        let model = main_window_weak.unwrap().get_buttons_grid();
        let mut tiles = model_grid_to_vec2d(model);
        expand_selection(&*game_config_cloned.borrow(), &position, &mut tiles);
        let model = vec2d_to_model_grid(tiles);
        main_window_weak.unwrap().set_buttons_grid(model);
    });

    main_window.run()
}
