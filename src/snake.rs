use std::{
	collections::{HashMap, VecDeque},
	hash::Hash,
};

#[derive(Debug, Clone)]
pub struct Snake {
	pub size: u16,
	pub position: Vector2D,
	pub direction: Direction,
	pub history: History<Vector2D>,
}

impl Default for Snake {
	fn default() -> Self {
		Self {
			size: 1,
			position: Vector2D { x: 0, y: 0 },
			direction: Direction::East,
			history: History::new(1),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vector2D {
	pub x: u16,
	pub y: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	North,
	East,
	South,
	West,
}

#[derive(Debug, Clone)]
pub struct History<T> {
	pub ordered_buffer: VecDeque<T>,
	pub unordered_buffer: HashMap<T, bool>,
	pub capacity: usize,
}

impl<T> History<T> {
	pub fn new(capacity: usize) -> Self {
		Self {
			ordered_buffer: VecDeque::with_capacity(capacity),
			unordered_buffer: HashMap::with_capacity(capacity),
			capacity,
		}
	}

	pub fn push(&mut self, value: T) -> Option<T>
	where
		T: Eq + Hash + Copy,
	{
		if self.ordered_buffer.len() >= self.capacity {
			let maybe_discarded_value = self.ordered_buffer.pop_front();
			if let Some(discarded_value) = maybe_discarded_value {
				self.unordered_buffer.remove(&discarded_value);
			}
			self.ordered_buffer.push_back(value);
			self.unordered_buffer.insert(value, true);
			return maybe_discarded_value;
		} else {
			self.ordered_buffer.push_back(value);
			self.unordered_buffer.insert(value, true);
			return None;
		}
	}

	pub fn increase_capacity(&mut self, new_capacity: usize) {
		if new_capacity > self.capacity {
			self.ordered_buffer.reserve(new_capacity);
			self.capacity = new_capacity;
		}
	}

	pub fn contains(&self, value: &T) -> bool
	where
		T: Eq + Hash + Copy,
	{
		self.unordered_buffer.contains_key(value)
	}
}
