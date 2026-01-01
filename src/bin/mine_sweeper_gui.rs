// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use modern_minesweeper::controller::{
    AboutDialog, GameConfig, GameDifficulty, GameState, MINE_VALUE, MainWindow, StateDialog,
    change_flag, change_visibility, check_win, clear_grid, expand_selection, fill_grid, new_grid,
    vec2d_to_model_grid, zero_pad,
};
use slint::ComponentHandle;
use std::{cell::RefCell, env, rc::Rc};

fn main() -> Result<(), slint::PlatformError> {
    unsafe {
        env::set_var("RUST_BACKTRACE", "1");
    }

    // Nullptr to State Dialog and About Dialog
    let state_dialog = Rc::new(RefCell::new(Option::<StateDialog>::None));
    let about_dialog = Rc::new(RefCell::new(Option::<AboutDialog>::None));

    // Global Configs
    let level = Rc::new(RefCell::new(GameDifficulty::Medium));
    let game_config = Rc::new(RefCell::new(GameConfig::new(*level.borrow())));

    // Empty Grid
    let tiles = Rc::new(RefCell::new(new_grid(&*game_config.borrow())));
    let model = vec2d_to_model_grid(&*tiles.borrow());
    let text_font_size = 28.0;
    let main_window = MainWindow::new()?;
    main_window.set_grid(model);
    main_window.set_state(GameState::Initial);
    main_window.set_mine_value(MINE_VALUE);
    main_window.set_flags(game_config.borrow().mine_count as i32);
    main_window.set_text_font_size(text_font_size);
    main_window.set_levels(GameDifficulty::create_model());
    main_window.invoke_initial_level((*level.borrow()).into());
    main_window.on_zero_pad(|number, length| zero_pad(number, length).into());

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
        main_window_weak
            .unwrap()
            .window()
            .dispatch_event(slint::platform::WindowEvent::CloseRequested);
    });

    // Restart Button
    let main_window_weak = main_window.as_weak();
    let tiles_cloned = tiles.clone();
    let game_config_cloned = game_config.clone();
    let state_dialog_cloned = state_dialog.clone();
    main_window.on_restart(move || {
        clear_grid(&mut *tiles_cloned.borrow_mut());
        let model = vec2d_to_model_grid(&*tiles_cloned.borrow());
        main_window_weak.unwrap().set_grid(model);
        main_window_weak.unwrap().set_state(GameState::Initial);
        main_window_weak
            .unwrap()
            .set_flags(game_config_cloned.borrow().mine_count as i32);
        main_window_weak.unwrap().invoke_reset_timer();
        // Close State Dialog if present
        if state_dialog_cloned.borrow().is_some() {
            let state_dialog = state_dialog_cloned.borrow();
            let state_dialog = state_dialog.as_ref().unwrap();
            state_dialog
                .window()
                .dispatch_event(slint::platform::WindowEvent::CloseRequested);
        }
    });

    // Expand Selection
    let main_window_weak = main_window.as_weak();
    let game_config_cloned = game_config.clone();
    let tiles_cloned = tiles.clone();
    let state_dialog_cloned = state_dialog.clone();
    main_window.on_expand_selection(move |position| {
        if let Some(_lose) = expand_selection(
            &*game_config_cloned.borrow(),
            &position,
            &mut *tiles_cloned.borrow_mut(),
        ) {
            main_window_weak.unwrap().set_state(GameState::Lose);
            // State Dialog
            create_state_dialog(state_dialog_cloned.clone(), text_font_size);
            let state_dialog = state_dialog_cloned.borrow();
            let state_dialog = state_dialog.as_ref().unwrap();
            state_dialog.set_state(GameState::Lose);
            state_dialog.show().unwrap();
        }
        let model = vec2d_to_model_grid(&*tiles_cloned.borrow());
        main_window_weak.unwrap().set_grid(model);
    });

    // Change Flag
    let tiles_cloned = tiles.clone();
    main_window.on_change_flag(move |position, flag| {
        change_flag(&mut *tiles_cloned.borrow_mut(), &position, flag);
    });

    // Change Visibility
    let tiles_cloned = tiles.clone();
    let main_window_weak = main_window.as_weak();
    let state_dialog_cloned = state_dialog.clone();
    main_window.on_change_visibility(move |position, visible| {
        change_visibility(&mut *tiles_cloned.borrow_mut(), &position, visible);
        let tiles_ref = &*tiles_cloned.borrow();
        let tile = &tiles_ref[position.row as usize][position.col as usize];
        if tile.value == MINE_VALUE {
            main_window_weak.unwrap().set_state(GameState::Lose);
            // State Dialog
            create_state_dialog(state_dialog_cloned.clone(), text_font_size);
            let state_dialog = state_dialog_cloned.borrow();
            let state_dialog = state_dialog.as_ref().unwrap();
            state_dialog.set_state(GameState::Lose);
            state_dialog.show().unwrap();
        }
    });

    // Win Condition
    let game_config_cloned = game_config.clone();
    let tiles_cloned = tiles.clone();
    let main_window_weak = main_window.as_weak();
    let state_dialog_cloned = state_dialog.clone();
    main_window.on_check_win(move || {
        if check_win(&*game_config_cloned.borrow(), &*tiles_cloned.borrow()) {
            main_window_weak.unwrap().set_state(GameState::Win);
            create_state_dialog(state_dialog_cloned.clone(), text_font_size);
            let state_dialog = state_dialog_cloned.borrow();
            let state_dialog = state_dialog.as_ref().unwrap();
            state_dialog.set_state(GameState::Win);
            state_dialog.show().unwrap();
        }
    });

    // Difficulty Changed
    let game_config_cloned = game_config.clone();
    let tiles_cloned = tiles.clone();
    let main_window_weak = main_window.as_weak();
    main_window.on_level_changed(move |index| {
        level.borrow_mut().clone_from(&index.into());
        game_config_cloned
            .borrow_mut()
            .clone_from(&GameConfig::new(*level.borrow()));
        tiles_cloned
            .borrow_mut()
            .clone_from(&new_grid(&*game_config_cloned.borrow()));
        let model = vec2d_to_model_grid(&*tiles_cloned.borrow());
        main_window_weak.unwrap().set_grid(model);
        main_window_weak
            .unwrap()
            .set_flags(game_config_cloned.borrow().mine_count as i32);
    });

    // About
    let about_dialog_cloned = about_dialog.clone();
    main_window.on_about(move || {
        create_about_dialog(about_dialog_cloned.clone());
        let about_dialog = about_dialog_cloned.borrow();
        let about_dialog = about_dialog.as_ref().unwrap();
        if about_dialog.window().is_visible() {
            about_dialog.hide().unwrap();
        }
        about_dialog.show().unwrap();
    });

    // Closing other windows
    let about_dialog_cloned = about_dialog.clone();
    let state_dialog_cloned = state_dialog.clone();
    main_window.window().on_close_requested(move || {
        // About Dialog
        let about_dialog = about_dialog_cloned.borrow();
        let about_dialog = about_dialog.as_ref();
        if let Some(about_dialog) = about_dialog {
            if about_dialog.window().is_visible() {
                about_dialog.hide().unwrap();
            }
        }
        // State Dialog
        let state_dialog = state_dialog_cloned.borrow();
        let state_dialog = state_dialog.as_ref();
        if let Some(state_dialog) = state_dialog {
            if state_dialog.window().is_visible() {
                state_dialog.hide().unwrap();
            }
        }
        // Closing finally
        slint::CloseRequestResponse::HideWindow
    });

    main_window.run()
}

fn create_state_dialog(state_dialog: Rc<RefCell<Option<StateDialog>>>, font_size: f32) {
    if state_dialog.borrow().is_none() {
        state_dialog.replace(Some(StateDialog::new().unwrap()));
        let state_dialog = state_dialog.borrow();
        let state_dialog = state_dialog.as_ref().unwrap();
        state_dialog.set_text_font_size(font_size);
        let state_dialog_weak = state_dialog.as_weak();
        state_dialog.on_close(move || {
            state_dialog_weak
                .unwrap()
                .window()
                .dispatch_event(slint::platform::WindowEvent::CloseRequested);
        });
    }
}

fn create_about_dialog(about_dialog: Rc<RefCell<Option<AboutDialog>>>) {
    if about_dialog.borrow().is_none() {
        about_dialog.replace(Some(AboutDialog::new().unwrap()));
        let about_dialog = about_dialog.borrow();
        let about_dialog = about_dialog.as_ref().unwrap();
        about_dialog.set_text_font_size(16.0);
        about_dialog.set_version(env!("CARGO_PKG_VERSION").into());
        about_dialog.set_home_page(env!("CARGO_PKG_REPOSITORY").into());
        about_dialog.set_license(env!("CARGO_PKG_LICENSE").into());
        let about_dialog_weak = about_dialog.as_weak();
        about_dialog.on_close(move || {
            about_dialog_weak
                .unwrap()
                .window()
                .dispatch_event(slint::platform::WindowEvent::CloseRequested);
        });
    }
}
