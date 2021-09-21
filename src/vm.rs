use crate::astraw::RawInstr;
use crate::astsoup::Block;
use std::io::{Read, Write};

struct VmMem {
	cell_vec: Vec<u8>,
	head: usize,
	interact_with_user: bool,
	input_stack: Vec<u8>,
	output_stack: Vec<u8>,
}

impl VmMem {
	fn new(input: Option<Vec<u8>>) -> VmMem {
		VmMem {
			cell_vec: Vec::new(),
			head: 0,
			interact_with_user: input.is_none(),
			input_stack: input.map_or(Vec::new(), |v| {
				v.into_iter().chain(std::iter::once(0)).rev().collect()
			}),
			output_stack: Vec::new(),
		}
	}

	fn get(&self, index: usize) -> u8 {
		self.cell_vec.get(index).copied().unwrap_or(0)
	}

	fn set(&mut self, index: usize, value: u8) {
		let len = self.cell_vec.len();
		if len <= index {
			self.cell_vec
				.extend(std::iter::repeat(0).take(index + 1 - len))
		}
		self.cell_vec[index] = value;
	}

	fn output_char_value(&mut self, char_value: u8) {
		if self.interact_with_user {
			print!("{}", char_value as char);
		}
		self.output_stack.push(char_value);
	}

	fn input_char_value(&mut self) -> u8 {
		match self.input_stack.pop() {
			Some(value) => value,
			None => {
				if self.interact_with_user {
					print!("\x1b[36m");
					std::io::stdout().flush().ok();
					self.input_stack.push(
						std::io::stdin()
							.bytes()
							.next()
							.transpose()
							.ok()
							.flatten()
							.unwrap_or(0),
					);
					print!("\x1b[39m");
					self.input_stack.pop().unwrap()
				} else {
					0
				}
			}
		}
	}
}

pub fn run_raw(instr_seq: Vec<RawInstr>, input: Option<Vec<u8>>) -> Vec<u8> {
	let mut m = VmMem::new(input);
	let mut instr_stack: Vec<RawInstr> = instr_seq.into_iter().rev().collect();
	while let Some(instr) = instr_stack.pop() {
		match &instr {
			RawInstr::Plus => m.set(m.head, m.get(m.head).wrapping_add(1)),
			RawInstr::Minus => m.set(m.head, m.get(m.head).wrapping_sub(1)),
			RawInstr::Left => {
				assert!(m.head >= 1);
				m.head -= 1;
			}
			RawInstr::Right => m.head += 1,
			RawInstr::Dot => {
				let char_value = m.get(m.head);
				m.output_char_value(char_value);
			}
			RawInstr::Comma => {
				let char_value = m.input_char_value();
				m.set(m.head, char_value);
			}
			RawInstr::BracketLoop(body) => {
				if m.get(m.head) != 0 {
					// The loop itself must be under its content.
					instr_stack.push(instr.clone());
					instr_stack.extend(body.iter().rev().cloned());
				}
			}
		}
	}
	if m.interact_with_user && m.output_stack.last().map_or(false, |&v| v != 10) {
		println!("");
	}
	m.output_stack
}

pub fn run_soup(instr_seq: Vec<Block>, input: Option<Vec<u8>>) -> Vec<u8> {
	let mut m = VmMem::new(input);
	let mut instr_stack: Vec<Block> = instr_seq.into_iter().rev().collect();
	while let Some(instr) = instr_stack.pop() {
		match &instr {
			Block::Soup {
				cell_deltas,
				head_delta,
			} => {
				for (relative_head, delta) in cell_deltas.iter() {
					let index = (m.head as isize + relative_head) as usize;
					let old_value: isize = m.get(index) as isize;
					let new_value = ((old_value + delta) as usize % 256) as u8;
					m.set(index, new_value);
				}
				m.head = (m.head as isize + head_delta) as usize;
			}
			Block::Output => {
				let char_value = m.get(m.head);
				m.output_char_value(char_value);
			}
			Block::Input => {
				let char_value = m.input_char_value();
				m.set(m.head, char_value);
			}
			Block::MultFixedLoop { cell_deltas } => {
				assert!(matches!(cell_deltas.get(&0), Some(-1)));
				let n = m.get(m.head) as isize;
				for (relative_head, delta) in cell_deltas.iter() {
					let index = (m.head as isize + relative_head) as usize;
					let old_value: isize = m.get(index) as isize;
					let new_value = ((old_value + delta * n) as usize % 256) as u8;
					m.set(index, new_value);
				}
				m.set(m.head, 0);
			}
			Block::SoupFixedLoop { cell_deltas } => {
				for (relative_head, delta) in cell_deltas.iter() {
					let index = (m.head as isize + relative_head) as usize;
					let old_value: isize = m.get(index) as isize;
					let new_value = ((old_value + delta) as usize % 256) as u8;
					m.set(index, new_value);
				}
				if m.get(m.head) != 0 {
					instr_stack.push(instr.clone());
				}
			}
			Block::SoupMovingLoop {
				cell_deltas,
				head_delta,
			} => {
				for (relative_head, delta) in cell_deltas.iter() {
					let index = (m.head as isize + relative_head) as usize;
					let old_value: isize = m.get(index) as isize;
					let new_value = ((old_value + delta) as usize % 256) as u8;
					m.set(index, new_value);
				}
				m.head = (m.head as isize + head_delta) as usize;
				if m.get(m.head) != 0 {
					instr_stack.push(instr.clone());
				}
			}
			Block::Loop(body) => {
				if m.get(m.head) != 0 {
					// The loop itself must be under its content.
					instr_stack.push(instr.clone());
					instr_stack.extend(body.iter().rev().cloned());
				}
			}
		}
	}
	if m.interact_with_user && m.output_stack.last().map_or(false, |&v| v != 10) {
		println!("");
	}
	m.output_stack
}
