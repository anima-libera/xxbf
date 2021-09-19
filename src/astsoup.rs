use std::collections::HashMap;

#[derive(Debug, Clone)]
enum CellModif {
	Delta(isize),
	CommaDelta(isize),
}

#[derive(Debug, Clone)]
struct Soup {
	cell_modif_map: HashMap<isize, CellModif>,
	head_delta: isize,
}

#[derive(Debug, Clone)]
pub enum SoupInstr {
	Soup(Soup),
	Dot,
	BracketLoop(Vec<SoupInstr>),
	FixedLoop(HashMap<isize, CellModif>),
}
