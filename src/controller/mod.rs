slint::include_modules!();

use meta_enum::{MetaEnum, ParseMetaEnumError};
use rand::{self, seq::index::sample_weighted};
use slint::{Model as _, ModelRc, SharedString, VecModel};

pub const MINE_VALUE: i32 = -1;

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub row_count: usize,
    pub col_count: usize,
    pub mine_count: usize,
}

#[derive(Debug, Clone, Copy, MetaEnum)]
pub enum GameDifficulty {
    Easy,
    Medium,
    Hard,
}

impl GameDifficulty {
    pub fn create_model() -> ModelRc<SharedString> {
        let model: Vec<_> = GameDifficulty::keys()
            .into_iter()
            .map(|key| key.into())
            .collect();
        VecModel::from_slice(&model)
    }
}

impl GameConfig {
    pub fn new(difficulty: GameDifficulty) -> Self {
        match difficulty {
            GameDifficulty::Easy => Self {
                row_count: 8,
                col_count: 8,
                mine_count: 10,
            },
            GameDifficulty::Medium => Self {
                row_count: 16,
                col_count: 16,
                mine_count: 40,
            },
            GameDifficulty::Hard => Self {
                row_count: 16,
                col_count: 30,
                mine_count: 99,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub struct GameOver;

pub fn new_grid(game_config: &GameConfig) -> Vec<Vec<Tile>> {
    let mut tiles = Vec::new();

    for _ in 0..game_config.row_count {
        let mut row_vec = Vec::new();
        for _ in 0..game_config.col_count {
            row_vec.push(Tile {
                value: 0,
                visible: false,
                flagged: false,
            });
        }
        tiles.push(row_vec);
    }
    tiles
}

pub fn clear_grid(tiles: &mut Vec<Vec<Tile>>) {
    for row in tiles.iter_mut() {
        for tile in row {
            tile.flagged = false;
            tile.visible = false;
            tile.value = 0;
        }
    }
}

pub fn fill_grid(game_config: &GameConfig, first_move: Position, tiles: &mut Vec<Vec<Tile>>) {
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
    let amount = game_config.mine_count as usize;
    let bombs_index = sample_weighted(&mut rng, length, weight, amount)
        .unwrap()
        .into_vec();

    // Setting The Bombs on the Grid
    for (i, row) in tiles.iter_mut().enumerate() {
        for (j, tile) in row.iter_mut().enumerate() {
            let value = match bombs_index.iter().find(|b| {
                **b == position_to_index(
                    game_config,
                    &Position {
                        row: i as i32,
                        col: j as i32,
                    },
                )
            }) {
                Some(_) => MINE_VALUE,
                None => 0,
            };
            tile.value = value;
        }
    }

    // Setting The Numbers
    for (i, row) in tiles.iter_mut().enumerate() {
        for (j, tile) in row.iter_mut().enumerate() {
            if tile.value != MINE_VALUE {
                let mut bombs = 0;
                let around = surronding_indicies(
                    game_config,
                    &Position {
                        row: i as i32,
                        col: j as i32,
                    },
                );
                for index in around {
                    if bombs_index.contains(&index) {
                        bombs += 1;
                    }
                }
                tile.value = bombs;
            }
        }
    }

    // Showing clicked Button and Around
    tiles[first_move.row as usize][first_move.col as usize].visible = true;
    let lost = expand_selection(game_config, &first_move, tiles);
    assert_eq!(lost, None);
}

pub fn model_grid_to_vec2d<T>(model: ModelRc<ModelRc<T>>) -> Vec<Vec<T>> {
    let model_vec: Vec<_> = model.iter().collect();

    let mut tiles_vec = Vec::new();
    for model in model_vec {
        let tiles: Vec<_> = model.iter().collect();
        tiles_vec.push(tiles);
    }
    tiles_vec
}

pub fn vec2d_to_model_grid(tiles: &Vec<Vec<Tile>>) -> ModelRc<ModelRc<Tile>> {
    let mut grid_model = Vec::new();
    for row in tiles {
        grid_model.push(VecModel::from_slice(&row));
    }
    VecModel::from_slice(&grid_model)
}

#[must_use]
pub fn expand_selection(
    game_config: &GameConfig,
    position: &Position,
    tiles: &mut Vec<Vec<Tile>>,
) -> Option<GameOver> {
    if tiles[position.row as usize][position.col as usize].value == 0 {
        let around = surronding_indicies(game_config, position);
        for index in around {
            let pos = index_to_position(game_config, index);
            let tile = &mut tiles[pos.row as usize][pos.col as usize];
            if !tile.flagged && !tile.visible {
                tile.visible = true;
                if tile.value == 0 {
                    if let Some(_) = expand_selection(game_config, &pos, tiles) {
                        return Some(GameOver);
                    }
                }
            }
        }
    } else {
        let around = surronding_indicies(game_config, position);
        let mut flags = 0;
        for index in around.iter() {
            let pos = index_to_position(game_config, *index);
            let tile = &tiles[pos.row as usize][pos.col as usize];
            if tile.flagged {
                flags += 1
            }
        }
        let tile = &tiles[position.row as usize][position.col as usize];
        if flags == tile.value {
            for index in around.iter() {
                let pos = index_to_position(game_config, *index);
                let tile = &mut tiles[pos.row as usize][pos.col as usize];
                if !tile.flagged && !tile.visible {
                    tile.visible = true;
                    if tile.value == MINE_VALUE {
                        return Some(GameOver);
                    }
                    if let Some(_) = expand_selection(game_config, &pos, tiles) {
                        return Some(GameOver);
                    }
                }
            }
        }
    }
    return None;
}

#[inline]
pub fn change_flag(tiles: &mut Vec<Vec<Tile>>, position: &Position, flag: bool) {
    tiles[position.row as usize][position.col as usize].flagged = flag;
}

#[inline]
pub fn change_visibility(tiles: &mut Vec<Vec<Tile>>, position: &Position, visible: bool) {
    tiles[position.row as usize][position.col as usize].visible = visible;
}

pub fn check_win(game_config: &GameConfig, tiles: &Vec<Vec<Tile>>) -> bool {
    let mut flags = 0;
    for row in tiles {
        for tile in row {
            if tile.flagged {
                flags += 1;
            } else if !tile.flagged && !tile.visible {
                return false;
            }
        }
    }
    game_config.mine_count == flags as usize
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

pub fn zero_pad(number: i32, length: i32) -> String {
    let mut value = number.to_string();
    let diff = length - value.len() as i32;
    if diff > 0 {
        value = "0".repeat(diff as usize) + value.as_str();
    }
    value
}
