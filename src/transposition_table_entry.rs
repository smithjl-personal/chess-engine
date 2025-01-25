use crate::r#move::Move;


#[derive(Clone)]
pub struct TranspositionTableEntry {
    pub zobrist_hash: u64,
    pub best_move: Option<Move>,
    pub depth: u32,
    pub evaluation: i64,
    pub node_type: TranspositionTableNodeType,
    pub age: u128,
}

#[derive(Clone)]
pub enum TranspositionTableNodeType {
    Exact,
    LowerBound,
    UpperBound,
}