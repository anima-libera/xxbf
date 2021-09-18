use crate::astraw::RawInstr;
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
				let output_value = m.get(m.head);
				if m.interact_with_user {
					print!("{}", output_value as char);
				}
				m.output_stack.push(output_value);
			}
			RawInstr::Comma => {
				let input_char_value = match m.input_stack.pop() {
					Some(value) => value,
					None => {
						if m.interact_with_user {
							//let mut input_line = String::new();
							print!("\x1b[36m");
							std::io::stdout().flush().ok();
							//std::io::stdin().read_line(&mut input_line).expect("h");
							m.input_stack.push(
								std::io::stdin()
									.bytes()
									.next()
									.transpose()
									.ok()
									.flatten()
									.unwrap_or(0),
							);
							print!("\x1b[39m");
							m.input_stack.pop().unwrap()
						} else {
							0
						}
					}
				};
				m.set(m.head, input_char_value);
			}
			RawInstr::BracketLoop(body) => {
				if m.get(m.head) != 0 {
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
