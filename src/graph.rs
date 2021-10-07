use crate::astsoup::SoupInstr;
use std::collections::HashMap;

enum BlockInstr {
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
	Loop(Vec<BlockInstr>),
}

type BlockId = u64;

enum Terminator {
	Goto(BlockId),
	Branch {
		if_zero: BlockId,
		if_non_zero: BlockId,
	},
}

struct Block {
	soup_instrs: Vec<BlockInstr>,
	terminator: Terminator,
}

struct Graph {
	blocks: HashMap<BlockId, Block>,
	next_id: BlockId,
}

fn grahify(raw_prog: &Vec<SoupInstr>) -> Graph {
	todo!()
}
