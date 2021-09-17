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

#[derive(Debug)]
struct RawProg {
	instr_seq: Vec<RawInstr>,
}
#[derive(Debug)]
enum RawInstr {
	Plus,
	Minus,
	Left,
	Right,
	Dot,
	Comma,
	BracketLoop(Vec<RawInstr>),
}

fn main() {
	dbg!(parse_instr_seq("+++[><.,[-]]-")).ok();
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
