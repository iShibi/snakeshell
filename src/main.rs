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

#[derive(Debug, Clone, Copy)]
struct GameState {
	pub score: u32,
}

impl Default for GameState {
	fn default() -> Self {
		Self { score: 0 }
	}
}

fn run_game_loop(cols: u16, rows: u16, stdout: &mut io::Stdout) -> io::Result<()> {
	// let mut score = 0;
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
	let mut snake = Snake::default();
	let mut is_game_over = false;
	let mut last_tick = Instant::now();
	let mut current_selection = 0;
	let game_over = "GAME OVER";
	let restart = "Restart";
	let exit = "Exit";
	'game_loop: loop {
		if let Ok(key_code) = reciever.try_recv() {
			match key_code {
				KeyCode::Esc => break,
				KeyCode::Char('w') => {
					if !is_game_over && snake.direction != Direction::South {
						snake.direction = Direction::North;
					} else {
						current_selection = 0;
					}
				}
				KeyCode::Char('d') => {
					if !is_game_over && snake.direction != Direction::West {
						snake.direction = Direction::East;
					}
				}
				KeyCode::Char('s') => {
					if !is_game_over && snake.direction != Direction::North {
						snake.direction = Direction::South;
					} else {
						current_selection = 1;
					}
				}
				KeyCode::Char('a') => {
					if !is_game_over && snake.direction != Direction::East {
						snake.direction = Direction::West;
					}
				}
				KeyCode::Enter => {
					if is_game_over && current_selection == 1 {
						break 'game_loop;
					} else if is_game_over && current_selection == 0 {
						snake = Snake::default();
						game_state.score = 0;
						is_game_over = false;
						current_selection = 0;
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
			if !is_game_over {
				let current_position = snake.position.clone();
				let maybe_tail = snake.history.push(current_position);
				match snake.direction {
					Direction::North => {
						if snake.position.y == 0 {
							snake.position.y = rows - 1;
						} else {
							snake.position.y -= 1;
						}
					}
					Direction::East => {
						snake.position.x = (snake.position.x + 1) % cols;
					}
					Direction::South => {
						snake.position.y = (snake.position.y + 1) % rows;
					}
					Direction::West => {
						if snake.position.x == 0 {
							snake.position.x = cols - 1;
						} else {
							snake.position.x -= 1;
						}
					}
				}
				if snake.history.contains(&snake.position) {
					print!("\x07"); // Makes a bell sound
					is_game_over = true;
					queue!(stdout, MoveTo(snake.position.x, snake.position.y), Print("!".red()))?;
					stdout.flush()?;
					continue;
				}
				if food_position == snake.position {
					game_state.score += 1;
					snake.size += 1;
					snake.history.increase_capacity(snake.size as usize);
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
						MoveTo(snake.position.x, snake.position.y),
						Print("*".green()),
						MoveTo(food_position.x, food_position.y),
						Print("@".red()),
					)?;
				} else {
					queue!(
						stdout,
						MoveTo(cols - score_ui.len() as u16, 0),
						Print(score_ui),
						MoveTo(snake.position.x, snake.position.y),
						Print("*".green()),
						MoveTo(food_position.x, food_position.y),
						Print("@".red()),
					)?;
				}
				stdout.flush()?;
			} else {
				let (center_x, center_y) = (((cols - game_over.len() as u16) / 2) - 1, (rows / 2) - 1);
				if current_selection == 0 {
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
				} else if current_selection == 1 {
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
