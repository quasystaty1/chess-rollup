use chess::{Board, ChessMove, Color, Game, GameResult};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GameState {
    pub game: Game,
    pub winner: Option<Color>, // None if the game is ongoing, Some(Color) if won
    pub moves: Vec<ChessMove>, // Keeps track of moves made in the game
}

use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub enum Transaction {
    StartGame { game_id: u32 },
    MakeMove { game_id: u32, move_san: String }, // SAN (Standard Algebraic Notation) for chess moves
}

#[derive(Debug, Clone)]
pub struct GameManager {
    pub games: HashMap<u32, GameState>, // Map between game index and GameState
    pub current_block_hash: Bytes,      // Hash of the current block
}

impl GameManager {
    // Create a new GameManager
    pub fn new(starting_hash: Bytes) -> Self {
        Self {
            games: HashMap::new(),
            current_block_hash: starting_hash,
        }
    }

    // Starts a new game and adds it to the games map
    pub fn start_new_game(&mut self, game_id: u32) {
        let game = Game::new();
        let game_state = GameState {
            game,
            winner: None,
            moves: Vec::new(),
        };
        self.games.insert(game_id, game_state);
    }

    // Attempt to make a move in the specified game
    pub fn make_move(&mut self, game_id: u32, move_str: &str) -> Result<(), String> {
        // Retrieve the game from the map
        let game_state = self.games.get_mut(&game_id).ok_or("Game not found")?;

        // Parse the move
        let chess_move = match ChessMove::from_san(&game_state.game.current_position(), move_str) {
            Ok(mv) => mv,
            Err(_) => return Err("Invalid move".into()),
        };

        // Make the move
        game_state.game.make_move(chess_move);
        game_state.moves.push(chess_move);

        // Check for game result
        if let Some(result) = game_state.game.result() {
            game_state.winner = match result {
                GameResult::WhiteCheckmates | GameResult::BlackResigns => Some(Color::White),
                GameResult::BlackCheckmates | GameResult::WhiteResigns => Some(Color::Black),
                GameResult::DrawAccepted | GameResult::Stalemate | GameResult::DrawDeclared => None, // Draw scenarios
            };
        }

        Ok(())
    }

    // Retrieve the status of a game
    pub fn game_status(&self, game_id: u32) -> Result<&GameState, String> {
        self.games.get(&game_id).ok_or("Game not found".into())
    }

    pub fn execute_transaction(
        &mut self,
        tx: Transaction,
        hasher: &mut Sha256,
    ) -> Result<(), String> {
        let tx_bytes = tx.encode();

        match tx {
            Transaction::StartGame { game_id } => {
                self.start_new_game(game_id);
                println!("starting new game: {}", game_id);
                hasher.update(&tx_bytes);
                Ok(())
            }
            Transaction::MakeMove { game_id, move_san } => {
                let result = self.make_move(game_id, &move_san);
                if result.is_ok() {
                    hasher.update(&tx_bytes);
                }
                result
            }
        }
    }

    pub fn process_transactions(&mut self, data: &Vec<Bytes>, current_hash: Bytes) -> Bytes {
        let mut hasher = Sha256::new();
        hasher.update(&current_hash);
        for encoded_tx in data {
            match Transaction::decode(encoded_tx.to_owned()) {
                Ok(tx) => {
                    if let Err(e) = self.execute_transaction(tx, &mut hasher) {
                        eprintln!("Failed to execute transaction: {}", e);
                    }
                }
                Err(e) => eprintln!("Failed to decode transaction: {}", e),
            }
        }
        hasher.finalize().into_iter().collect()
    }

    pub fn finalize_block_hash(&mut self, hasher: Sha256) -> Bytes {
        // Finalize the block hash and save it as the current block's hash
        let final_hash = hasher.finalize();
        let final_hash_bytes = Bytes::copy_from_slice(&final_hash);
        self.current_block_hash = final_hash_bytes.clone();
        final_hash_bytes
    }
}

impl Transaction {
    // Encode the transaction into bytes
    pub fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();

        match self {
            Transaction::StartGame { game_id } => {
                buffer.put_u8(0); // Indicator for StartGame
                buffer.put_u32(*game_id);
            }
            Transaction::MakeMove { game_id, move_san } => {
                buffer.put_u8(1); // Indicator for MakeMove
                buffer.put_u32(*game_id);
                buffer.put_u32(move_san.len() as u32);
                buffer.put_slice(move_san.as_bytes());
            }
        }

        buffer.freeze()
    }

    // Decode bytes back into a transaction
    pub fn decode(mut data: Bytes) -> Result<Self, String> {
        if data.remaining() < 5 {
            return Err("Data too short".into());
        }

        let tx_type = data.get_u8();
        let game_id = data.get_u32();

        match tx_type {
            0 => Ok(Transaction::StartGame { game_id }),
            1 => {
                if data.remaining() < 4 {
                    return Err("Data too short for move length".into());
                }
                let move_len = data.get_u32() as usize;

                if data.remaining() < move_len {
                    return Err("Data too short for move".into());
                }

                let move_san = String::from_utf8(data.split_to(move_len).to_vec())
                    .map_err(|_| "Invalid UTF-8 in move string")?;

                Ok(Transaction::MakeMove { game_id, move_san })
            }
            _ => Err("Unknown transaction type".into()),
        }
    }
}
