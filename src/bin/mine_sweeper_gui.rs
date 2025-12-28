// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use modern_minesweeper::controller::{
    AboutDialog, GameConfig, GameState, MINE_VALUE, MainWindow, check_win, clear_grid,
    expand_selection, fill_grid, new_grid, vec2d_to_model_grid,
};
use slint::ComponentHandle;
use std::{cell::RefCell, env, rc::Rc};

fn main() -> Result<(), slint::PlatformError> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }
    let main_window = MainWindow::new()?;

    // Global Configs
    let game_config = Rc::new(RefCell::new(GameConfig::default()));

    // Empty Grid
    let tiles = Rc::new(RefCell::new(new_grid(&*game_config.borrow())));
    let model = vec2d_to_model_grid(&*tiles.borrow());
    main_window.set_grid(model);
    main_window.set_state(GameState::Initial);
    main_window.set_mine_value(MINE_VALUE);

    // First Move Occured
    let main_window_weak = main_window.as_weak();
    let game_config_cloned = game_config.clone();
    let tiles_cloned = tiles.clone();
    main_window.on_first_move_occured(move |position| {
        fill_grid(
            &*game_config_cloned.borrow(),
            position,
            &mut *tiles_cloned.borrow_mut(),
        );
        let model = vec2d_to_model_grid(&*tiles_cloned.borrow());
        main_window_weak.unwrap().set_grid(model);
        main_window_weak.unwrap().set_state(GameState::Normal);
    });

    // Quit Button
    let main_window_weak = main_window.as_weak();
    main_window.on_close(move || {
        main_window_weak.unwrap().hide().unwrap();
    });

    // Restart Button
    let main_window_weak = main_window.as_weak();
    let tiles_cloned = tiles.clone();
    main_window.on_restart(move || {
        clear_grid(&mut *tiles_cloned.borrow_mut());
        let model = vec2d_to_model_grid(&*tiles_cloned.borrow());
        main_window_weak.unwrap().set_grid(model);
        main_window_weak.unwrap().set_state(GameState::Initial);
    });

    // Expand Selection
    let main_window_weak = main_window.as_weak();
    let game_config_cloned = game_config.clone();
    let tiles_cloned = tiles.clone();
    main_window.on_expand_selection(move |position| {
        if let Some(_) = expand_selection(
            &*game_config_cloned.borrow(),
            &position,
            &mut *tiles_cloned.borrow_mut(),
        ) {
            main_window_weak.unwrap().set_state(GameState::Lose);
        }
        let model = vec2d_to_model_grid(&*tiles_cloned.borrow());
        main_window_weak.unwrap().set_grid(model);
    });

    // Change Flag
    let tiles_cloned = tiles.clone();
    main_window.on_change_flag(move |position, flag| {
        let tiles_mut = &mut *tiles_cloned.borrow_mut();
        tiles_mut[position.row as usize][position.col as usize].flagged = flag;
    });

    // Change Visibility
    let tiles_cloned = tiles.clone();
    let main_window_weak = main_window.as_weak();
    main_window.on_change_visibility(move |position, visible| {
        let tiles_mut = &mut *tiles_cloned.borrow_mut();
        let tile = &mut tiles_mut[position.row as usize][position.col as usize];
        tile.visible = visible;
        if tile.value == -1 {
            main_window_weak.unwrap().set_state(GameState::Lose);
        }
    });

    // Win Condition
    let game_config_cloned = game_config.clone();
    let tiles_cloned = tiles.clone();
    let main_window_weak = main_window.as_weak();
    main_window.on_check_win(move || {
        if check_win(&*game_config_cloned.borrow(), &*tiles_cloned.borrow()) {
            main_window_weak.unwrap().set_state(GameState::Win);
        }
    });

    // About
    main_window.on_about(|| {
        if let Ok(about) = AboutDialog::new() {
            about.show().unwrap();
            let about_weak = about.as_weak();
            about.on_close(move || {
                about_weak.unwrap().hide().unwrap();
            });
        }
    });

    main_window.run()
}
