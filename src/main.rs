pub mod snake;

use crossterm::cursor::{Hide, MoveTo, SetCursorStyle, Show};
use crossterm::event::{self, Event, KeyCode};
use crossterm::style::{Attribute, Attributes, Print, SetAttributes, Stylize};
use crossterm::terminal::{
	Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, SetTitle, disable_raw_mode, enable_raw_mode, size,
};
use crossterm::{execute, queue};
use rand::Rng;
use snake::{Direction, Snake, Vector2D};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

fn main() -> io::Result<()> {
	let (cols, rows) = size()?;
	let mut stdout = io::stdout();
	enable_raw_mode()?;
	execute!(stdout, EnterAlternateScreen)?;
	queue!(
		stdout,
		Hide,
		Clear(ClearType::All),
		Clear(ClearType::Purge),
		MoveTo(0, 0),
		SetTitle("Snake Game")
	)?;
	stdout.flush()?;
	run_game_loop(cols, rows, &mut stdout)?;
	execute!(stdout, LeaveAlternateScreen)?;
	queue!(stdout, SetCursorStyle::DefaultUserShape, Show)?;
	stdout.flush()?;
	disable_raw_mode()?;
	Ok(())
}

#[derive(Debug, Clone)]
struct GameState {
	pub score: u32,
	pub is_game_over: bool,
	pub snake: Snake,
	pub current_selection: u8,
}

impl Default for GameState {
	fn default() -> Self {
		Self {
			score: 0,
			is_game_over: false,
			snake: Snake::default(),
			current_selection: 0,
		}
	}
}

fn run_game_loop(cols: u16, rows: u16, stdout: &mut io::Stdout) -> io::Result<()> {
	let mut game_state = GameState::default();
	let mut rng = rand::rng();
	let mut food_position = Vector2D {
		x: rng.random_range(0..cols),
		y: rng.random_range(0..rows),
	};
	let (sender, reciever) = mpsc::channel();
	thread::spawn(move || {
		loop {
			if let Event::Key(key_event) = event::read().unwrap() {
				if let Err(_) = sender.send(key_event.code) {}
			}
		}
	});
	let game_over = "GAME OVER";
	let restart = "Restart";
	let exit = "Exit";
	let mut last_tick = Instant::now();
	'game_loop: loop {
		if let Ok(key_code) = reciever.try_recv() {
			match key_code {
				KeyCode::Esc => break,
				KeyCode::Char('w') => {
					if !game_state.is_game_over && game_state.snake.direction != Direction::South {
						game_state.snake.direction = Direction::North;
					} else {
						game_state.current_selection = 0;
					}
				}
				KeyCode::Char('d') => {
					if !game_state.is_game_over && game_state.snake.direction != Direction::West {
						game_state.snake.direction = Direction::East;
					}
				}
				KeyCode::Char('s') => {
					if !game_state.is_game_over && game_state.snake.direction != Direction::North {
						game_state.snake.direction = Direction::South;
					} else {
						game_state.current_selection = 1;
					}
				}
				KeyCode::Char('a') => {
					if !game_state.is_game_over && game_state.snake.direction != Direction::East {
						game_state.snake.direction = Direction::West;
					}
				}
				KeyCode::Enter => {
					if game_state.is_game_over && game_state.current_selection == 1 {
						break 'game_loop;
					} else if game_state.is_game_over && game_state.current_selection == 0 {
						game_state = GameState::default();
						food_position = Vector2D {
							x: rng.random_range(0..cols),
							y: rng.random_range(0..rows),
						};
						queue!(
							stdout,
							Hide,
							Clear(ClearType::All),
							Clear(ClearType::Purge),
							MoveTo(0, 0)
						)?;
						stdout.flush()?;
					}
				}
				_ => (),
			}
		}
		if last_tick.elapsed() >= Duration::from_millis(100) {
			if !game_state.is_game_over {
				let current_position = game_state.snake.position.clone();
				let maybe_tail = game_state.snake.history.push(current_position);
				match game_state.snake.direction {
					Direction::North => {
						if game_state.snake.position.y == 0 {
							game_state.snake.position.y = rows - 1;
						} else {
							game_state.snake.position.y -= 1;
						}
					}
					Direction::East => {
						game_state.snake.position.x = (game_state.snake.position.x + 1) % cols;
					}
					Direction::South => {
						game_state.snake.position.y = (game_state.snake.position.y + 1) % rows;
					}
					Direction::West => {
						if game_state.snake.position.x == 0 {
							game_state.snake.position.x = cols - 1;
						} else {
							game_state.snake.position.x -= 1;
						}
					}
				}
				if game_state.snake.history.contains(&game_state.snake.position) {
					print!("\x07"); // Makes a bell sound
					game_state.is_game_over = true;
					queue!(
						stdout,
						MoveTo(game_state.snake.position.x, game_state.snake.position.y),
						Print("!".red())
					)?;
					stdout.flush()?;
					continue;
				}
				if food_position == game_state.snake.position {
					game_state.score += 1;
					game_state.snake.size += 1;
					game_state.snake.history.increase_capacity(game_state.snake.size as usize);
					food_position = Vector2D {
						x: rng.random_range(0..cols),
						y: rng.random_range(0..rows),
					};
				}
				let score_ui = format!("Score: {score}", score = game_state.score);
				if let Some(Vector2D { x, y }) = maybe_tail {
					queue!(
						stdout,
						MoveTo(x, y),
						Print(" "),
						MoveTo(cols - score_ui.len() as u16, 0),
						Print(score_ui),
						MoveTo(game_state.snake.position.x, game_state.snake.position.y),
						Print("*".green()),
						MoveTo(food_position.x, food_position.y),
						Print("@".red()),
					)?;
				} else {
					queue!(
						stdout,
						MoveTo(cols - score_ui.len() as u16, 0),
						Print(score_ui),
						MoveTo(game_state.snake.position.x, game_state.snake.position.y),
						Print("*".green()),
						MoveTo(food_position.x, food_position.y),
						Print("@".red()),
					)?;
				}
				stdout.flush()?;
			} else {
				let (center_x, center_y) = (((cols - game_over.len() as u16) / 2) - 1, (rows / 2) - 1);
				if game_state.current_selection == 0 {
					queue!(
						stdout,
						MoveTo(center_x, center_y),
						SetAttributes(Attributes::none().with(Attribute::Bold)),
						Print(game_over),
						SetAttributes(Attributes::none().with(Attribute::NormalIntensity)),
						MoveTo(center_x, center_y + 4),
						Print(restart),
						MoveTo(center_x, center_y + 5),
						Print(exit),
						MoveTo(center_x - 1, center_y + 4),
						Show,
						SetCursorStyle::SteadyBlock
					)?;
				} else if game_state.current_selection == 1 {
					queue!(
						stdout,
						MoveTo(center_x, center_y),
						SetAttributes(Attributes::none().with(Attribute::Bold)),
						Print(game_over),
						SetAttributes(Attributes::none().with(Attribute::NormalIntensity)),
						MoveTo(center_x, center_y + 4),
						Print(restart),
						MoveTo(center_x, center_y + 5),
						Print(exit),
						MoveTo(center_x - 1, center_y + 5),
						Show,
						SetCursorStyle::SteadyBlock
					)?;
				}
				stdout.flush()?;
			}
			last_tick = Instant::now();
		}
	}
	Ok(())
}
