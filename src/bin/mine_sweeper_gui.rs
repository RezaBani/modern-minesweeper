use std::{cell::RefCell, env, rc::Rc};

use minesweeper_slint::mine_sweeper_ui::{GameConfig, MainWindow, fill_grid, new_grid};
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
        let buttons_grid = fill_grid(&*game_config_cloned.borrow(), position);
        let buttons_grid_model = VecModel::from_slice(&buttons_grid);
        main_window_weak
            .unwrap()
            .set_buttons_grid(buttons_grid_model);
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

    main_window.run()
}
