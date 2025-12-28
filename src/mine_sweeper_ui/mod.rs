mod main_window;

pub use main_window::*;
use rand::{self, seq::index::sample_weighted};
use slint::{Model, ModelRc, VecModel};

#[derive(Debug, Clone)]
pub struct GameConfig {
    row_count: usize,
    col_count: usize,
    bomb_count: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            row_count: 10,
            col_count: 15,
            bomb_count: 50,
        }
    }
}

pub fn new_grid(game_config: &GameConfig) -> Vec<ModelRc<Tile>> {
    let mut buttons_grid = Vec::new();

    for row in 0..game_config.row_count {
        let mut row_vec = Vec::new();
        for col in 0..game_config.col_count {
            row_vec.push(Tile {
                position: Position {
                    row: row as i32,
                    col: col as i32,
                },
                value: 0,
                visible: false,
                flagged: false,
            });
        }
        buttons_grid.push(VecModel::from_slice(&row_vec));
    }
    buttons_grid
}

pub fn fill_grid(game_config: &GameConfig, first_move: Position) -> Vec<Vec<Tile>> {
    // Making First Button not be a bomb
    let mut zero_weights = surronding_indicies(game_config, &first_move);
    zero_weights.push(position_to_index(game_config, &first_move));
    let weight = |index| {
        if zero_weights.contains(&index) {
            0.0
        } else {
            0.5
        }
    };

    // Getting the random bombs
    let mut rng = rand::rng();
    let length = (game_config.row_count * game_config.col_count) as usize;
    let amount = game_config.bomb_count as usize;
    let bombs_index = sample_weighted(&mut rng, length, weight, amount)
        .unwrap()
        .into_vec();

    // Setting The Bombs on the Grid
    let mut buttons_grid = Vec::new();
    for row in 0..game_config.row_count {
        let mut row_vec = Vec::new();
        for col in 0..game_config.col_count {
            let position = Position {
                row: row as i32,
                col: col as i32,
            };
            let value = match bombs_index
                .iter()
                .find(|b| **b == position_to_index(game_config, &position))
            {
                Some(_) => -1,
                None => 0,
            };
            row_vec.push(Tile {
                position,
                value,
                visible: false,
                flagged: false,
            });
        }
        buttons_grid.push(row_vec);
    }

    // Setting The Numbers
    for row_vec in &mut buttons_grid {
        for item in row_vec {
            if item.value != -1 {
                let mut bombs = 0;
                let around = surronding_indicies(game_config, &item.position);
                for tile in around {
                    if bombs_index.contains(&tile) {
                        bombs += 1;
                    }
                }
                item.value = bombs;
            }
        }
    }

    // Showing clicked Button and Around
    buttons_grid[first_move.row as usize][first_move.col as usize].visible = true;
    expand_selection(game_config, &first_move, &mut buttons_grid);
    buttons_grid
}

pub fn model_grid_to_vec2d(model: ModelRc<ModelRc<Tile>>) -> Vec<Vec<Tile>> {
    let model_vec: Vec<_> = model.iter().collect();

    let mut tiles_vec = Vec::new();
    for model in model_vec {
        let tiles: Vec<_> = model.iter().collect();
        tiles_vec.push(tiles);
    }
    tiles_vec
}

pub fn vec2d_to_model_grid(tiles: Vec<Vec<Tile>>) -> ModelRc<ModelRc<Tile>> {
    let mut grid_model = Vec::new();
    for row in tiles {
        grid_model.push(VecModel::from_slice(&row));
    }
    VecModel::from_slice(&grid_model)
}

pub fn expand_selection(game_config: &GameConfig, position: &Position, tiles: &mut Vec<Vec<Tile>>) {
    if tiles[position.row as usize][position.col as usize].value == 0 {
        let around = surronding_indicies(game_config, position);
        for index in around {
            let pos = index_to_position(game_config, index);
            let tile = &mut tiles[pos.row as usize][pos.col as usize];
            if !tile.flagged && !tile.visible {
                tile.visible = true;
                if tile.value == 0 {
                    expand_selection(game_config, &tile.position.clone(), tiles);
                }
            }
        }
    }
}

#[inline]
fn position_to_index(game_config: &GameConfig, position: &Position) -> usize {
    position.row as usize * game_config.col_count + position.col as usize
}

fn index_to_position(game_config: &GameConfig, index: usize) -> Position {
    let row = (index / game_config.col_count) as i32;
    let col = (index % game_config.col_count) as i32;
    Position { row, col }
}

fn surronding_indicies(game_config: &GameConfig, position: &Position) -> Vec<usize> {
    // For this function to work we assume that grid is at least 2x2
    assert!(game_config.row_count > 1);
    assert!(game_config.col_count > 1);

    let mut indicies = Vec::new();

    if position.row > 0 {
        // We are not top row
        // Above
        indicies.push(position_to_index(
            game_config,
            &Position {
                row: position.row - 1,
                col: position.col,
            },
        ));

        if position.col > 0 {
            // Above Left
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row - 1,
                    col: position.col - 1,
                },
            ));

            // Left
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row,
                    col: position.col - 1,
                },
            ));
        }

        if position.col < game_config.col_count as i32 - 1 {
            // Above Right
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row - 1,
                    col: position.col + 1,
                },
            ));

            // Right
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row,
                    col: position.col + 1,
                },
            ));
        }

        if position.row < game_config.row_count as i32 - 1 {
            // Below
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row + 1,
                    col: position.col,
                },
            ));

            if position.col > 0 {
                // Below Left
                indicies.push(position_to_index(
                    game_config,
                    &Position {
                        row: position.row + 1,
                        col: position.col - 1,
                    },
                ));
            }

            if position.col < game_config.col_count as i32 - 1 {
                // Below Right
                indicies.push(position_to_index(
                    game_config,
                    &Position {
                        row: position.row + 1,
                        col: position.col + 1,
                    },
                ));
            }
        }
    } else {
        indicies.push(position_to_index(
            game_config,
            &Position {
                row: position.row + 1,
                col: position.col,
            },
        ));

        if position.col > 0 {
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row,
                    col: position.col - 1,
                },
            ));

            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row + 1,
                    col: position.col - 1,
                },
            ));
        }

        if position.col < game_config.col_count as i32 - 1 {
            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row,
                    col: position.col + 1,
                },
            ));

            indicies.push(position_to_index(
                game_config,
                &Position {
                    row: position.row + 1,
                    col: position.col + 1,
                },
            ));
        }
    }
    indicies
}
