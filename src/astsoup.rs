use crate::astraw::RawInstr;
use std::collections::HashMap;
use std::collections::HashSet;

enum CellInfoForward {
	Value(u8),
	ValueSet(HashSet<u8>),
	Unknown,
}

enum CellInfoBackward {
	Unused,
	Used,
	Unresolved,
}

enum TapeSliceInfoForward {
	Cell(CellInfoForward),
	CellGroup(Vec<CellInfoForward>),
	Slice {
		element: Box<TapeSliceInfoForward>,
		length: Option<usize>,
	},
}

struct TapeInfoForward {
	tape_slice_vec: Vec<TapeSliceInfoForward>,
	head: usize,
}

#[derive(Debug, Clone)]
pub enum Block {
	Soup {
		cell_deltas: HashMap<isize, isize>,
		head_delta: isize,
	},
	Output,
	Input,
	MultFixedLoop {
		// Cell delta on head is -1 here.
		cell_deltas: HashMap<isize, isize>,
	},
	SoupFixedLoop {
		cell_deltas: HashMap<isize, isize>,
	},
	SoupMovingLoop {
		cell_deltas: HashMap<isize, isize>,
		head_delta: isize,
	},
	Loop(Vec<Block>),
}

pub fn soupify(raw_prog: &Vec<RawInstr>) -> Vec<Block> {
	let mut soup_prog: Vec<Block> = Vec::new();
	fn top_must_be_soup(soup_prog: &mut Vec<Block>) {
		if !matches!(soup_prog.last(), Some(Block::Soup { .. })) {
			soup_prog.push(Block::Soup {
				cell_deltas: HashMap::new(),
				head_delta: 0,
			});
		}
	}

	for raw_instr in raw_prog {
		match raw_instr {
			RawInstr::Plus | RawInstr::Minus | RawInstr::Left | RawInstr::Right => {
				top_must_be_soup(&mut soup_prog);
				if let Some(&mut Block::Soup {
					ref mut cell_deltas,
					ref mut head_delta,
				}) = soup_prog.last_mut()
				{
					match raw_instr {
						RawInstr::Plus => *cell_deltas.entry(*head_delta).or_insert(0) += 1,
						RawInstr::Minus => *cell_deltas.entry(*head_delta).or_insert(0) -= 1,
						RawInstr::Left => *head_delta -= 1,
						RawInstr::Right => *head_delta += 1,
						_ => unreachable!(),
					}
				} else {
					unreachable!()
				}
			}
			RawInstr::Dot => soup_prog.push(Block::Output),
			RawInstr::Comma => soup_prog.push(Block::Input),
			RawInstr::BracketLoop(raw_instr_vec) => {
				let body = soupify(raw_instr_vec);
				if body.len() == 1 && matches!(body[0], Block::Soup { .. }) {
					match &body[0] {
						Block::Soup {
							cell_deltas,
							head_delta,
						} => {
							if *head_delta == 0 && *cell_deltas.get(&0).unwrap_or(&0) == -1 {
								soup_prog.push(Block::MultFixedLoop {
									cell_deltas: cell_deltas.clone(),
								});
							} else if *head_delta == 0 {
								soup_prog.push(Block::SoupFixedLoop {
									cell_deltas: cell_deltas.clone(),
								});
							} else {
								soup_prog.push(Block::SoupMovingLoop {
									cell_deltas: cell_deltas.clone(),
									head_delta: *head_delta,
								});
							}
						}
						_ => unreachable!(),
					}
				} else {
					soup_prog.push(Block::Loop(body));
				}
			}
		}
	}
	soup_prog
}
