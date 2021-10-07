use crate::astraw::RawInstr;
use crate::astsoup::SoupInstr;
use std::collections::HashMap;

struct TranspiledC {
	code: String,
	indent_level: u32,
}

impl TranspiledC {
	fn new() -> TranspiledC {
		TranspiledC {
			code: String::new(),
			indent_level: 0,
		}
	}

	fn emit_line(&mut self, line_content: &str) {
		self.code
			.extend(std::iter::repeat("\t").take(self.indent_level as usize));
		self.code.extend(line_content.chars());
		self.code.extend("\n".chars());
	}

	fn emit_indent(&mut self) {
		self.indent_level += 1;
	}
	fn emit_unindent(&mut self) {
		self.indent_level -= 1;
	}

	fn emit_header(&mut self) {
		assert!(self.code.len() == 0);
		assert!(self.indent_level == 0);
		self.emit_line("#include <stdio.h>");
		self.emit_line("int main(void)");
		self.emit_line("{");
		self.emit_indent();
		self.emit_line("unsigned char m[30000] = {0};");
		self.emit_line("unsigned int h = 0;");
	}

	fn emit_footer(&mut self) {
		self.emit_line("return 0;");
		self.emit_unindent();
		self.emit_line("}");
		assert!(self.indent_level == 0);
	}

	fn emit_raw_instr_seq(&mut self, instr_seq: Vec<RawInstr>) {
		for instr in instr_seq {
			match instr {
				RawInstr::Plus => self.emit_line("m[h]++;"),
				RawInstr::Minus => self.emit_line("m[h]--;"),
				RawInstr::Left => self.emit_line("h--;"),
				RawInstr::Right => self.emit_line("h++;"),
				RawInstr::Dot => self.emit_line("putchar(m[h]);"),
				RawInstr::Comma => self.emit_line("m[h] = getchar();"),
				RawInstr::BracketLoop(body) => {
					self.emit_line("while (m[h])");
					self.emit_line("{");
					self.emit_indent();
					self.emit_raw_instr_seq(body);
					self.emit_unindent();
					self.emit_line("}");
				}
			}
		}
	}

	fn emit_soup_instr_seq(&mut self, instr_seq: Vec<SoupInstr>) {
		for instr in instr_seq {
			match instr {
				SoupInstr::Soup {
					cell_deltas,
					head_delta,
				} => {
					let cell_deltas = sort_cell_deltas(cell_deltas);
					for (relative_head, delta) in cell_deltas {
						self.emit_line(&format!("m[{}] += {};", h(relative_head), delta));
					}
					if head_delta != 0 {
						self.emit_line(&format!("h += {};", head_delta));
					}
				}
				SoupInstr::Output => self.emit_line("putchar(m[h]);"),
				SoupInstr::Input => self.emit_line("m[h] = getchar();"),
				SoupInstr::MultFixedLoop { cell_deltas } => {
					assert!(matches!(cell_deltas.get(&0), Some(-1)));
					let cell_deltas = sort_cell_deltas(cell_deltas);
					for (relative_head, delta) in cell_deltas.iter() {
						if *relative_head == 0 {
							continue;
						}
						self.emit_line(&format!("m[{}] += m[h] * {};", h(*relative_head), delta));
					}
					self.emit_line(&format!("m[h] = 0;"));
				}
				SoupInstr::SoupFixedLoop { cell_deltas } => {
					self.emit_line("while (m[h])");
					self.emit_line("{");
					self.emit_indent();
					let cell_deltas = sort_cell_deltas(cell_deltas);
					for (relative_head, delta) in cell_deltas {
						self.emit_line(&format!("m[{}] += {};", h(relative_head), delta));
					}
					self.emit_unindent();
					self.emit_line("}");
				}
				SoupInstr::SoupMovingLoop {
					cell_deltas,
					head_delta,
				} => {
					self.emit_line("while (m[h])");
					self.emit_line("{");
					self.emit_indent();
					let cell_deltas = sort_cell_deltas(cell_deltas);
					for (relative_head, delta) in cell_deltas {
						self.emit_line(&format!("m[{}] += {};", h(relative_head), delta));
					}
					self.emit_line(&format!("h += {};", head_delta));
					self.emit_unindent();
					self.emit_line("}");
				}
				SoupInstr::Loop(body) => {
					self.emit_line("while (m[h])");
					self.emit_line("{");
					self.emit_indent();
					self.emit_soup_instr_seq(body);
					self.emit_unindent();
					self.emit_line("}");
				}
			}
		}
	}
}

pub fn transpile_raw_to_c(instr_seq: Vec<RawInstr>) -> String {
	let mut transpiled = TranspiledC::new();
	transpiled.emit_header();
	transpiled.emit_raw_instr_seq(instr_seq);
	transpiled.emit_footer();
	transpiled.code
}

pub fn transpile_soup_to_c(instr_seq: Vec<SoupInstr>) -> String {
	let mut transpiled = TranspiledC::new();
	transpiled.emit_header();
	transpiled.emit_soup_instr_seq(instr_seq);
	transpiled.emit_footer();
	transpiled.code
}

// Head relative positions are sorted for output readability purposes
fn sort_cell_deltas(cell_deltas: HashMap<isize, isize>) -> Vec<(isize, isize)> {
	let mut cell_deltas = cell_deltas
		.iter()
		.map(|(&k, &v)| (k, v))
		.collect::<Vec<(isize, isize)>>();
	cell_deltas.sort_by(|(ka, _), (kb, _)| ka.partial_cmp(kb).unwrap());
	cell_deltas
}

fn h(relative_head: isize) -> String {
	if relative_head == 0 {
		"h".to_owned()
	} else {
		format!("h + {}", relative_head)
	}
}
