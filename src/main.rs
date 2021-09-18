#[derive(Debug, Clone)]
enum RawInstr {
	Plus,
	Minus,
	Left,
	Right,
	Dot,
	Comma,
	BracketLoop(Vec<RawInstr>),
}

fn parse_instr_seq(src_code: &str) -> Result<Vec<RawInstr>, Vec<ParsingError>> {
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
enum ParsingError {
	UnmatchedOpeningBracket { pos: usize },
	UnmatchedClosingBracket { pos: usize },
}

impl ParsingError {
	fn print(self, src_code: &str, src_code_name: Option<&str>, ansi_escape_codes: bool) {
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

fn transpile_raw_to_c(instr_seq: Vec<RawInstr>) -> String {
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

fn run_raw(instr_seq: Vec<RawInstr>, input: Option<Vec<u8>>) -> Vec<u8> {
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
							let mut input_line = String::new();
							std::io::stdin().read_line(&mut input_line).expect("h");
							m.input_stack =
								input_line.bytes().chain(std::iter::once(0)).rev().collect();
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

enum WhatToDo {
	Interpret {
		input: Option<String>,
	},
	Compile {
		target: CompileTarget,
		dst_file_path: Option<String>,
	},
}

enum CompileTarget {
	C,
}

enum SrcSettings {
	Src(String),
	FilePath(String),
	None,
}

struct Settings {
	path: Option<String>,
	src: SrcSettings,
	what_to_do: WhatToDo,
}

impl Settings {
	fn from_cmdline_args() -> Settings {
		let mut args = std::env::args();
		let mut settings = Settings {
			path: args.next(),
			src: SrcSettings::None,
			what_to_do: WhatToDo::Interpret { input: None },
		};
		while let Some(arg) = args.next() {
			if arg == "-s" || arg == "--src" {
				settings.src = SrcSettings::Src(args.next().unwrap())
			} else if arg == "-f" || arg == "--src-file" {
				settings.src = SrcSettings::FilePath(args.next().unwrap())
			} else if arg == "-c" || arg == "--compile" {
				settings.what_to_do = WhatToDo::Compile {
					target: CompileTarget::C,
					dst_file_path: args.next(),
				};
			} else if let WhatToDo::Interpret { ref mut input } = settings.what_to_do {
				if arg == "-i" || arg == "--input" {
					*input = args.next();
				} else {
					panic!("unknown cmdline argument `{}` (for interpretation)", arg);
				}
			} else if let WhatToDo::Compile {
				ref mut dst_file_path,
				..
			} = settings.what_to_do
			{
				if arg == "-o" || arg == "--output-file" {
					*dst_file_path = args.next();
				} else {
					panic!("unknown cmdline argument `{}` (for compilation)", arg);
				}
			} else {
				panic!("hhh");
			}
		}
		settings
	}
}

fn main() {
	let settings = Settings::from_cmdline_args();

	let src_code = match settings.src {
		SrcSettings::Src(src_code) => src_code,
		SrcSettings::FilePath(src_file_path) => std::fs::read_to_string(src_file_path).expect("h"),
		SrcSettings::None => {
			println!("No source code, nothing to do.");
			return;
		}
	};

	let parsing_result = parse_instr_seq(&src_code);
	let prog = match parsing_result {
		Ok(prog) => prog,
		Err(error_vec) => {
			for error in error_vec {
				error.print(&src_code, None, true);
			}
			return;
		}
	};

	match settings.what_to_do {
		WhatToDo::Interpret { input } => {
			let interact_with_user = input.is_some();
			let input = input.map(|s| s.bytes().collect());
			let output = run_raw(prog, input);
			let output_string: String = output.iter().map(|&x| x as char).collect();
			if interact_with_user {
				println!("{}", output_string);
			}
		}
		WhatToDo::Compile {
			target,
			dst_file_path,
		} => {
			let output_code = match target {
				CompileTarget::C => transpile_raw_to_c(prog),
			};
			if let Some(dst_file_path) = dst_file_path {
				std::fs::write(dst_file_path, output_code).expect("h");
			} else {
				print!("{}", output_code);
			}
		}
	}
}

/*
use std::collections::HashMap;

type Prog = Vec<Block>;

#[derive(Debug, Clone)]
enum Block {
	BeginExecutionRaw,
	RawPlus,
	RawMinus,
	RawLeft,
	RawRight,
	RawDot,
	RawComma,
	RawBracketLoop(Box<Block>),
	EndExecutionRaw,
	Sequence(Vec<Block>),
	Pot(Pot),
}

#[derive(Debug, Clone)]
enum CellPot {
	Known(u8),
	Set(u8),
	Delta(i32),
}

#[derive(Debug, Clone)]
struct Pot {
	cells: HashMap<i32, CellPot>,
	selection_delta: i32,
}

impl Block {
	fn is_raw_instruction(&self) -> bool {
		match &self {
			Block::RawPlus
			| Block::RawMinus
			| Block::RawLeft
			| Block::RawRight
			| Block::RawDot
			| Block::RawComma => true,
			_ => false,
		}
	}
	fn is_user_interaction(&self) -> bool {
		match &self {
			Block::RawDot | Block::RawComma => true,
			_ => false,
		}
	}
}

impl Block {
	fn to_pot(&self) -> Option<Block> {
		match self {
			Block::RawPlus => Some(Block::Pot(Pot {
				cells: {
					let mut map = HashMap::new();
					map.insert(0, CellPot::Delta(1));
					map
				},
				selection_delta: 0,
			})),
			Block::RawMinus => Some(Block::Pot(Pot {
				cells: {
					let mut map = HashMap::new();
					map.insert(0, CellPot::Delta(-1));
					map
				},
				selection_delta: 0,
			})),
			Block::RawLeft => Some(Block::Pot(Pot {
				cells: HashMap::new(),
				selection_delta: -1,
			})),
			Block::RawRight => Some(Block::Pot(Pot {
				cells: HashMap::new(),
				selection_delta: 1,
			})),
			_ => None,
		}
	}
	fn potify_if_possible(&mut self) {
		let potified_opt = self.to_pot();
		if let Some(potified) = potified_opt {
			*self = potified;
		}
	}
}

impl From<Prog> for Block {
	fn from(prog: Prog) -> Block {
		Block::Sequence(prog)
	}
}

struct ParsingSuccess {
	prog: Prog,
}
#[derive(Debug)]
enum ParsingError {
	UnmatchedOpeningBracket(usize),
	UnmatchedClosingBracket(usize),
}
#[derive(Debug)]
struct ParsingErrorPack {
	errors: Vec<ParsingError>,
}
type ParsingResult = Result<ParsingSuccess, ParsingErrorPack>;

impl std::fmt::Display for ParsingErrorPack {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "parsing error") // TODO
	}
}

fn parse(src_code: &str, is_whole_program: bool) -> ParsingResult {
	enum BeingParsedProg {
		Root(Prog),
		BracketLoop(Prog, usize), // The uszie is the opening bracket position.
	}
	impl BeingParsedProg {
		fn append(&mut self, block: Block) {
			match self {
				BeingParsedProg::Root(prog) => prog.push(block),
				BeingParsedProg::BracketLoop(prog, _) => prog.push(block),
			}
		}
	}

	struct ProgStack {
		stack: Vec<BeingParsedProg>,
	}
	impl ProgStack {
		fn new() -> ProgStack {
			ProgStack {
				stack: vec![BeingParsedProg::Root(Vec::new())],
			}
		}
		fn top(&mut self) -> &mut BeingParsedProg {
			self.stack.last_mut().unwrap()
		}
		fn open_bracket_loop(&mut self, opening_bracket_index: usize) {
			self.stack.push(BeingParsedProg::BracketLoop(
				Vec::new(),
				opening_bracket_index,
			));
		}
		fn close_bracket_loop(
			&mut self,
			errors: &mut Vec<ParsingError>,
			closing_bracket_index: usize,
		) {
			if self.stack.len() < 2 {
				errors.push(ParsingError::UnmatchedClosingBracket(closing_bracket_index))
			} else {
				let closing_prog = match self.stack.pop().unwrap() {
					BeingParsedProg::BracketLoop(prog, _) => prog,
					_ => panic!("bfxx parser bug"),
				};
				let closing_block = Box::new(Block::Sequence(closing_prog));
				self.top().append(Block::RawBracketLoop(closing_block));
			}
		}
		fn to_final_prog(mut self) -> Prog {
			assert!(self.stack.len() == 1);
			match self.stack.pop().unwrap() {
				BeingParsedProg::Root(prog) => prog,
				BeingParsedProg::BracketLoop(prog, _) => prog,
			}
		}
	}

	let mut errors: Vec<ParsingError> = Vec::new();

	let mut prog_stack: ProgStack = ProgStack::new();
	if is_whole_program {
		prog_stack.top().append(Block::BeginExecutionRaw);
	}
	let mut i = 0;
	while i < src_code.as_bytes().len() {
		let c_index = i;
		let c = src_code.as_bytes()[c_index] as char;
		i += 1;
		if c == '[' {
			prog_stack.open_bracket_loop(c_index);
		} else if c == ']' {
			prog_stack.close_bracket_loop(&mut errors, c_index);
		} else {
			match c {
				'+' => prog_stack.top().append(Block::RawPlus),
				'-' => prog_stack.top().append(Block::RawMinus),
				'<' => prog_stack.top().append(Block::RawLeft),
				'>' => prog_stack.top().append(Block::RawRight),
				'.' => prog_stack.top().append(Block::RawDot),
				',' => prog_stack.top().append(Block::RawComma),
				_ => (),
			}
		}
	}
	while prog_stack.stack.len() > 1 {
		let opening_bracket_index = match prog_stack.stack.pop().unwrap() {
			BeingParsedProg::BracketLoop(_, opening_bracket_index) => opening_bracket_index,
			_ => panic!("bfxx parser bug"),
		};
		errors.push(ParsingError::UnmatchedOpeningBracket(opening_bracket_index))
	}
	if is_whole_program {
		prog_stack.top().append(Block::EndExecutionRaw);
	}
	if errors.is_empty() {
		Ok(ParsingSuccess {
			prog: prog_stack.to_final_prog(),
		})
	} else {
		Err(ParsingErrorPack { errors })
	}
}

impl ParsingError {
	fn print(self, src_code: &str, src_code_name: Option<&str>, ansi_escape_codes: bool) {
		let error_index = match self {
			ParsingError::UnmatchedClosingBracket(error_index) => error_index,
			ParsingError::UnmatchedOpeningBracket(error_index) => error_index,
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
			ParsingError::UnmatchedClosingBracket(_) => "Unmatched closing bracket",
			ParsingError::UnmatchedOpeningBracket(_) => "Unmatched opening bracket",
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

enum MaybeMergeResult {
	Merged(Block),
	NotMerged(Block, Block),
}

fn maybe_merge(left: Block, right: Block) -> MaybeMergeResult {
	match (&left, &right) {
		/*
		(Block::Pot(left), Block::Pot(right)) => MaybeMergeResult::Merged(Block::Pot(Pot {
			cells: {
				let map = left.cells;
				for (index_rel, right_cell_pot) in right.cells {
					match map.get_mut(&index_rel) {
						None => {
							map.insert(index_rel, right_cell_pot);
						}
						Some(left_cell_pot) => {
							// TODO: all the cases xd
							todo!()

							// NOTE: becareful with the fact that index_rel for the
							// right is relative to the position AFTER the delta from
							// the left
						}
					}
				}
				map
			},
			selection_delta: left.selection_delta + right.selection_delta,
		})),
		*/
		(left, Block::EndExecutionRaw) if !left.is_user_interaction() => {
			MaybeMergeResult::Merged(Block::EndExecutionRaw)
		}
		(_, _) => MaybeMergeResult::NotMerged(left, right),
	}
}

fn potify_block(block: &mut Block) {
	match block {
		Block::RawBracketLoop(sub_block) => potify_block(sub_block),
		Block::Sequence(block_vec) => potify_prog(block_vec),
		block => block.potify_if_possible(),
	}
}

fn potify_prog(prog: &mut Prog) {
	for block in prog {
		potify_block(block);
	}
}

// TODO: Optimize.
fn optimize(mut prog: Prog) -> Prog {
	// Potify everything.
	potify_prog(&mut prog);

	// For each succession of two blocks, try to merge them.
	let mut pass_number = 1;
	loop {
		println!("Optimization pass {}", pass_number);
		let mut did_a_merge_happened = false;
		let mut i = 0;
		while i + 1 < prog.len() {
			let left = prog.remove(i);
			let right = prog.remove(i);
			let maybe_merge_result = maybe_merge(left, right);
			match maybe_merge_result {
				MaybeMergeResult::Merged(block) => {
					prog.insert(i, block);
					did_a_merge_happened = true;
				}
				MaybeMergeResult::NotMerged(left, right) => {
					prog.insert(i, left);
					prog.insert(i + 1, right);
					i += 1;
				}
			}
		}
		if !did_a_merge_happened {
			break;
		} else {
			pass_number += 1;
		}
	}
	prog
}

#[derive(Debug)]
struct Vm {
	mem: Vec<u8>,
	selected_index: usize,
	input: String,
	input_index: usize,
	output: String,
}

impl Vm {
	fn new() -> Vm {
		Vm {
			mem: Vec::new(),
			selected_index: 0,
			input: String::new(),
			input_index: 0,
			output: String::new(),
		}
	}

	fn get_mut_cell(&mut self, index: usize) -> &mut u8 {
		if index >= self.mem.len() {
			self.mem.resize(index + 1, 0);
		}
		self.mem.get_mut(index).unwrap()
	}
	fn get_cell(&mut self, index: usize) -> &u8 {
		if index >= self.mem.len() {
			self.mem.resize(index + 1, 0);
		}
		self.mem.get(index).unwrap()
	}

	fn get_mut_sel(&mut self) -> &mut u8 {
		self.get_mut_cell(self.selected_index)
	}
	fn get_sel(&mut self) -> &u8 {
		self.get_cell(self.selected_index)
	}

	fn run_block(&mut self, block: &Block, input: &str) {
		let mut block_stack: Vec<&Block> = vec![block];
		self.input += input;
		loop {
			let current_block = match block_stack.pop() {
				Some(block) => block,
				None => break,
			};
			match current_block {
				Block::BeginExecutionRaw => (),
				Block::RawPlus => *self.get_mut_sel() = self.get_sel().overflowing_add(1).0,
				Block::RawMinus => *self.get_mut_sel() = self.get_sel().overflowing_sub(1).0,
				Block::RawLeft => {
					assert!(self.selected_index >= 1);
					self.selected_index -= 1;
				}
				Block::RawRight => {
					self.selected_index += 1;
				}
				Block::RawDot => {
					let c = *self.get_sel() as char;
					self.output.push(c);
				}
				Block::RawComma => {
					let c = *self.input.as_bytes().get(self.input_index).unwrap_or(&0);
					self.input_index += 1;
					*self.get_mut_sel() = c;
				}
				Block::RawBracketLoop(sub_block) => {
					if *self.get_sel() != 0 {
						block_stack.push(current_block);
						block_stack.push(sub_block);
					}
				}
				Block::Sequence(block_vec) => {
					block_stack.extend(block_vec.iter().rev());
				}
				Block::EndExecutionRaw => (),
				Block::Pot(pot) => {
					for (rel_index, cell_pot) in pot.cells.iter() {
						let index = (self.selected_index as isize + *rel_index as isize) as usize;
						match cell_pot {
							CellPot::Known(value) => assert!(self.get_cell(index) == value),
							CellPot::Set(value) => {
								*self.get_mut_cell(index) = *value;
							}
							CellPot::Delta(delta) => {
								if *delta < 0 {
									*self.get_mut_cell(index) = self
										.get_cell(index)
										.overflowing_sub(((-*delta) % 256) as u8)
										.0;
								} else {
									*self.get_mut_cell(index) = self
										.get_cell(index)
										.overflowing_add((*delta % 256) as u8)
										.0;
								}
							}
						}
					}
					self.selected_index =
						(self.selected_index as isize + pot.selection_delta as isize) as usize;
				}
			}
		}
	}
}

fn emit_indent(transpiled: &mut String, indent_level: u32) {
	transpiled.extend(std::iter::repeat("\t").take(indent_level as usize));
}

fn emit_line(transpiled: &mut String, indent_level: u32, line_content: &str) {
	emit_indent(transpiled, indent_level);
	transpiled.push_str(line_content);
	transpiled.push('\n');
}

fn emit_block_vec(transpiled: &mut String, indent_level: u32, block_vec: &Vec<Block>) {
	for sub_block in block_vec {
		emit_block(transpiled, indent_level, sub_block);
	}
}

fn emit_block(transpiled: &mut String, indent_level: u32, block: &Block) {
	match &block {
		Block::BeginExecutionRaw => {
			emit_line(transpiled, indent_level, "unsigned char mem[30000] = {0};");
			emit_line(transpiled, indent_level, "unsigned int head = 0;");
		}
		Block::RawPlus => {
			emit_line(transpiled, indent_level, "mem[head]++;");
		}
		Block::RawMinus => {
			emit_line(transpiled, indent_level, "mem[head]--;");
		}
		Block::RawLeft => {
			emit_line(transpiled, indent_level, "assert(head >= 1);");
			emit_line(transpiled, indent_level, "head--;");
		}
		Block::RawRight => {
			emit_line(transpiled, indent_level, "head++;");
		}
		Block::RawDot => {
			emit_line(transpiled, indent_level, "putchar(mem[head]);");
		}
		Block::RawComma => {
			emit_line(transpiled, indent_level, "mem[head] = getchar();");
		}
		Block::RawBracketLoop(sub_block) => {
			emit_line(transpiled, indent_level, "while (mem[head] != 0)");
			emit_line(transpiled, indent_level, "{");
			emit_block(transpiled, indent_level + 1, sub_block);
			emit_line(transpiled, indent_level, "}");
		}
		Block::EndExecutionRaw => {
			emit_line(transpiled, indent_level, "return 0;");
		}
		Block::Sequence(sub_block_vec) => {
			emit_block_vec(transpiled, indent_level, sub_block_vec);
		}
		Block::Pot(pot) => {
			for (rel_index, cell_pot) in pot.cells.iter() {
				match cell_pot {
					CellPot::Known(value) => (),
					CellPot::Set(value) => {
						emit_line(
							transpiled,
							indent_level,
							&format!("mem[head + {}] = {};", rel_index, value),
						);
					}
					CellPot::Delta(delta) => {
						emit_line(
							transpiled,
							indent_level,
							&format!("mem[head + {}] += {};", rel_index, delta),
						);
					}
				}
			}
			if pot.selection_delta != 0 {
				emit_line(
					transpiled,
					indent_level,
					&format!("head += {};", pot.selection_delta),
				);
			}
		}
	}
}

fn emit_prog(transpiled: &mut String, prog: &Prog) {
	let indent_level = 0;
	emit_line(transpiled, indent_level, "#include <stdio.h>");
	emit_line(transpiled, indent_level, "#include <assert.h>");
	emit_line(transpiled, indent_level, "int main(void)");
	emit_line(transpiled, indent_level, "{");
	emit_block_vec(transpiled, indent_level + 1, prog);
	emit_line(transpiled, indent_level, "}");
	emit_line(transpiled, indent_level, "");
}

/*
const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
const HELLO_WORLD_2: &str = "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.";
const TEST_1: &str = "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.";
const TEST_2: &str = "++++++++++[>++++++++++<-]>---.";
const SIERPINSKI: &str = "
[sierpinski.b -- display Sierpinski triangle
(c) 2016 Daniel B. Cristofani
http://brainfuck.org/]

++++++++[>+>++++<<-]>++>>+<[-[>>+<<-]+>>]>+[
	-<<<[
		->[+[-]+>++>>>-<<]<[<]>>++++++[<<+++++>>-]+<<++.[-]<<
	]>.>+[>>]>+
]

[Shows an ASCII representation of the Sierpinski triangle
(iteration 5).]
";
*/

fn main() {
	let src_code = String::from("++++++++++[->++++++++++<]>.---");
	let parsing_result = parse(&src_code, true);
	let prog = match parsing_result {
		ParsingResult::Ok(ParsingSuccess { prog }) => prog,
		ParsingResult::Err(ParsingErrorPack { errors }) => {
			for error in errors {
				error.print(&src_code, None, true);
			}
			return;
		}
	};

	let prog = optimize(prog);

	dbg!(&prog);

	let mut vm = Vm::new();
	vm.run_block(&prog.clone().into(), "");
	println!("{}", vm.output);

	let mut transpiled = String::new();
	emit_prog(&mut transpiled, &prog);
	print!("{}", transpiled);
}
*/
