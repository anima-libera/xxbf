use crate::astraw::RawInstr;

pub fn parse_instr_seq(src_code: &str) -> Result<Vec<RawInstr>, Vec<ParsingError>> {
	// A scope is either the whole program or a bracket loop and its content.
	// Only the bottom scope isn't a bracket loop (and thus doesn't have an opening bracket pos),
	// this bottom scope should always be there (such design is for convenience).
	struct Scope {
		opening_bracket_pos: Option<usize>,
		instr_seq: Vec<RawInstr>,
	}
	struct ScopeStack(Vec<Scope>);
	impl ScopeStack {
		fn top_instr_seq(&mut self) -> &mut Vec<RawInstr> {
			&mut self.0.last_mut().unwrap().instr_seq
		}
	}
	let mut scope_stack: ScopeStack = ScopeStack(vec![Scope {
		opening_bracket_pos: None,
		instr_seq: Vec::new(),
	}]);

	let mut errors: Vec<ParsingError> = Vec::new();

	for (pos, c) in src_code.char_indices() {
		match c {
			'+' => scope_stack.top_instr_seq().push(RawInstr::Plus),
			'-' => scope_stack.top_instr_seq().push(RawInstr::Minus),
			'<' => scope_stack.top_instr_seq().push(RawInstr::Left),
			'>' => scope_stack.top_instr_seq().push(RawInstr::Right),
			'.' => scope_stack.top_instr_seq().push(RawInstr::Dot),
			',' => scope_stack.top_instr_seq().push(RawInstr::Comma),
			'[' => scope_stack.0.push(Scope {
				opening_bracket_pos: Some(pos),
				instr_seq: Vec::new(),
			}),
			']' => {
				if scope_stack.0.len() >= 2 {
					let poped_instr_seq = scope_stack.0.pop().unwrap().instr_seq;
					scope_stack
						.top_instr_seq()
						.push(RawInstr::BracketLoop(poped_instr_seq));
				} else {
					errors.push(ParsingError::UnmatchedClosingBracket { pos });
				}
			}
			_ => (),
		}
	}

	assert!(scope_stack.0.len() != 0);
	while scope_stack.0.len() >= 2 {
		// The use of `.remove(1)` here instead of `.pop().unwrap()` ensures that errors are
		// sorted according to their `pos`.
		let opening_bracket_pos = scope_stack.0.remove(1).opening_bracket_pos.unwrap();
		errors.push(ParsingError::UnmatchedOpeningBracket {
			pos: opening_bracket_pos,
		});
	}

	if errors.is_empty() {
		assert!(scope_stack.0.len() == 1);
		Ok(scope_stack.0.pop().unwrap().instr_seq)
	} else {
		Err(errors)
	}
}

#[derive(Debug)]
pub enum ParsingError {
	UnmatchedOpeningBracket { pos: usize },
	UnmatchedClosingBracket { pos: usize },
}

impl ParsingError {
	pub fn print(self, src_code: &str, src_code_name: Option<&str>, ansi_escape_codes: bool) {
		let error_index = match self {
			ParsingError::UnmatchedOpeningBracket { pos } => pos,
			ParsingError::UnmatchedClosingBracket { pos } => pos,
		};

		// Find the line that contains the error.
		let mut line_number = 1;
		let mut line_start_index = 0;
		let mut line_end_index = src_code.len() - 1;
		let mut this_is_the_line = false;
		for (index, c) in src_code.char_indices() {
			if index == error_index {
				this_is_the_line = true;
			}
			if c == '\n' {
				if this_is_the_line {
					line_end_index = index - 1;
					break;
				} else {
					line_number += 1;
					line_start_index = index + 1;
					continue;
				}
			}
		}
		let line_number = line_number;
		let line = &src_code[line_start_index..=line_end_index];
		let inline_error_index = error_index - line_start_index;

		let bold_on = if ansi_escape_codes { "\x1b[1m" } else { "" };
		let bold_off = if ansi_escape_codes { "\x1b[22m" } else { "" };
		let color_red = if ansi_escape_codes { "\x1b[31m" } else { "" };
		let color_light_red = if ansi_escape_codes { "\x1b[91m" } else { "" };
		let color_blue = if ansi_escape_codes { "\x1b[34m" } else { "" };
		let color_cyan = if ansi_escape_codes { "\x1b[36m" } else { "" };
		let color_off = if ansi_escape_codes { "\x1b[39m" } else { "" };

		// Print the head line of the error message.
		let error_variant_as_string = match self {
			ParsingError::UnmatchedClosingBracket { pos: _ } => "Unmatched closing bracket",
			ParsingError::UnmatchedOpeningBracket { pos: _ } => "Unmatched opening bracket",
		};
		println!(
			"{}{}Parsing error{} on line {} column {}{}: {}{}",
			bold_on,
			color_red,
			color_off,
			line_number,
			inline_error_index + 1,
			match src_code_name {
				Some(name) => format!(" of {}", name),
				None => "".to_owned(),
			},
			error_variant_as_string,
			bold_off
		);

		// Print the involved line of code with some formatting, and save the printed column of the
		// error character to be able to print a carret exactly under it.
		let mut initial_whitespace = true;
		let mut carret_column = 0;
		for (inline_index, c) in line.char_indices() {
			// Skip initial whitespace.
			if initial_whitespace && c.is_whitespace() {
				continue;
			} else {
				initial_whitespace = false;
			}

			if c == '\t' {
				// Make sure that tabs are manually extended to a fixed number of columns.
				print!("    ");
				if inline_index < inline_error_index {
					carret_column += 4;
				}
			} else if inline_index == inline_error_index {
				// Print the erroneous character with emphasis if possible.
				print!(
					"{}{}{}{}{}",
					bold_on, color_light_red, c, color_off, bold_off
				);
			} else if matches!(c, '+' | '-' | '<' | '>' | '[' | ']' | '.' | ',')
				|| c.is_whitespace()
			{
				// Print instruction characters normally.
				print!("{}", c);
				if inline_index < inline_error_index {
					carret_column += 1;
				}
			} else {
				// Print comment characters in a different way if possible.
				print!("{}{}{}", color_blue, c, color_off);
				if inline_index < inline_error_index {
					carret_column += 1;
				}
			}
		}
		let carret_column = carret_column;

		// Print a carret under the erroneous character.
		println!("");
		for _ in 0..carret_column {
			print!(" ");
		}
		println!("{}{}^ here{}{}", bold_on, color_cyan, color_off, bold_off);
	}
}
