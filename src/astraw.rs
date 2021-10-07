#[derive(Debug, Clone)]
pub enum RawInstr {
	Plus,
	Minus,
	Left,
	Right,
	Output,
	Input,
	BracketLoop(Vec<RawInstr>),
}
