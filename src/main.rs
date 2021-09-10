type Prog = Vec<Block>;

#[derive(Debug)]
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
}

impl From<Prog> for Block {
	fn from(prog: Prog) -> Block {
		Block::Sequence(prog)
	}
}

struct ParsingSuccess {
	prog: Prog,
	warnings: (),
}
#[derive(Debug)]
struct ParsingError {
	errors: (),
}
type ParsingResult = Result<ParsingSuccess, ParsingError>;

impl std::fmt::Display for ParsingError {
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
		fn close_bracket_loop(&mut self, closing_bracket_index: usize) {
			assert!(self.stack.len() >= 2);
			let closing_prog = match self.stack.pop().unwrap() {
				BeingParsedProg::BracketLoop(prog, _) => prog,
				_ => panic!("ono"),
			};
			let closing_block = Box::new(Block::Sequence(closing_prog));
			self.top().append(Block::RawBracketLoop(closing_block));
		}
		fn to_final_prog(mut self) -> Prog {
			assert!(self.stack.len() == 1);
			match self.stack.pop().unwrap() {
				BeingParsedProg::Root(prog) => prog,
				BeingParsedProg::BracketLoop(prog, _) => prog,
			}
		}
	}

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
			prog_stack.close_bracket_loop(c_index);
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
	assert!(prog_stack.stack.len() == 1);
	if is_whole_program {
		prog_stack.top().append(Block::EndExecutionRaw);
	}
	Ok(ParsingSuccess {
		prog: prog_stack.to_final_prog(),
		warnings: (),
	})
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
	fn get_mut_selected_cell(&mut self) -> &mut u8 {
		self.get_mut_cell(self.selected_index)
	}

	fn run_block(&mut self, block: &Block) {
		let mut block_stack: Vec<&Block> = vec![block];
		loop {
			let current_block = match block_stack.pop() {
				Some(block) => block,
				None => break,
			};
			match current_block {
				Block::BeginExecutionRaw => (),
				Block::RawPlus => {
					*self.get_mut_selected_cell() =
						self.get_mut_selected_cell().overflowing_add(1).0
				}
				Block::RawMinus => {
					*self.get_mut_selected_cell() =
						self.get_mut_selected_cell().overflowing_sub(1).0
				}
				Block::RawLeft => {
					assert!(self.selected_index >= 1);
					self.selected_index -= 1;
				}
				Block::RawRight => {
					self.selected_index += 1;
				}
				Block::RawDot => {
					let c = *self.get_mut_selected_cell() as char;
					self.output.push(c);
				}
				Block::RawComma => {
					let c = *self.input.as_bytes().get(self.input_index).unwrap_or(&0);
					self.input_index += 1;
					*self.get_mut_selected_cell() = c;
				}
				Block::RawBracketLoop(sub_block) => {
					if *self.get_mut_selected_cell() != 0 {
						block_stack.push(current_block);
						block_stack.push(sub_block);
					}
				}
				Block::Sequence(block_vec) => {
					block_stack.extend(block_vec.iter().rev());
				}
				Block::EndExecutionRaw => (),
			}
		}
	}
}

fn main() {
	const HELLO_WORLD: &str = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
	const HELLO_WORLD_2: &str = "++++++++++[>+++++++>++++++++++>+++>+<<<<-]>++.>+.+++++++..+++.>++.<<+++++++++++++++.>.+++.------.--------.>+.>.";
	const TEST_1: &str = "+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++.";
	const TEST_2: &str = "++++++++++[>++++++++++<-]>---.";
	const SIERPINSKI: &str = "[sierpinski.b -- display Sierpinski triangle
	(c) 2016 Daniel B. Cristofani
	http://brainfuck.org/]
	
	++++++++[>+>++++<<-]>++>>+<[-[>>+<<-]+>>]>+[
		-<<<[
			->[+[-]+>++>>>-<<]<[<]>>++++++[<<+++++>>-]+<<++.[-]<<
		]>.>+[>>]>+
	]
	
	[Shows an ASCII representation of the Sierpinski triangle
	(iteration 5).]";

	let prog = parse(&String::from(SIERPINSKI), true).unwrap().prog;
	dbg!(&prog);

	let mut vm = Vm::new();
	vm.run_block(&prog.into());
	println!("{}", vm.output);
}
