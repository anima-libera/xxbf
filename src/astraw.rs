#[derive(Debug, Clone)]
pub enum RawInstr {
	Plus,
	Minus,
	Left,
	Right,
	Dot,
	Comma,
	BracketLoop(Vec<RawInstr>),
}
