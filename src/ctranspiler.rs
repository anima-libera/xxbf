use crate::astraw::RawInstr;

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
					self.indent_level += 1;
					self.emit_raw_instr_seq(body);
					self.indent_level -= 1;
					self.emit_line("}");
				}
			}
		}
	}
}

pub fn transpile_raw_to_c(instr_seq: Vec<RawInstr>) -> String {
	let mut transpiled = TranspiledC::new();
	transpiled.emit_line("#include <stdio.h>");
	transpiled.emit_line("int main(void)");
	transpiled.emit_line("{");
	transpiled.indent_level += 1;
	transpiled.emit_line("unsigned char m[30000] = {0};");
	transpiled.emit_line("unsigned int h = 0;");
	transpiled.emit_raw_instr_seq(instr_seq);
	transpiled.emit_line("return 0;");
	transpiled.indent_level -= 1;
	transpiled.emit_line("}");
	assert!(transpiled.indent_level == 0);
	transpiled.code
}
