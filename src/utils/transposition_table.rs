use super::piece_move::Move;

/// An entry into the transposition table.
pub struct TTEntry {
    /// The hashed board state.
    pub zobrist_key: u64,
    /// The depth of the search.
    pub depth: usize,
    /// The evaluation of the position.
    pub evaluation: i32,
    /// The type of evaluation from the search.
    pub evaluation_type: EvaluationType,
    /// The best move from the search.
    pub best_move: Option<Move>
}

/// The type of evaluation from a search.
pub enum EvaluationType {
    /// An exact evaluation from `eval::evaluate_board(...)`.
    Exact,
    /// The real evaluation is, at most, equal to the stored evaluation.
    UpperBound,
    /// The real evaluation must be at least equal to the stored evaluation.
    LowerBound
}

/// A struct holding the transposition table entries, as well
/// as the maximum size for the transposition table.
pub struct TranspositionTable {
    /// The entries in the table.
    table: Vec<Option<TTEntry>>,
    /// The number of entries/buckets in the table.
    buckets: usize
}

impl TranspositionTable {
    /// Creates a new transposition table.
    /// 
    /// NOTE: Bucket size doesn't need to be a power of two
    /// since the indexer does not use a modulo.
    pub fn new(buckets: usize) -> Self {
        Self {
            table: Vec::with_capacity(buckets),
            buckets
        }
    }

    /// Creates a new transposition table from a size in megabytes.
    pub fn from_mb(size: usize) -> Self {
        let desired_size = size * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();
        let buckets = desired_size / entry_size;

        TranspositionTable::new(buckets)
    }

    /// Resizes the transposition table.
    pub fn resize(&mut self, buckets: usize) {
        self.table.resize_with(buckets, || None);
    }

    /// Resizes the transposition table from megabytes.
    pub fn resize_mb(&mut self, size: usize) {
        let desired_size = size * 1024 * 1024;
        let entry_size = std::mem::size_of::<TTEntry>();
        let buckets = desired_size / entry_size;

        self.resize(buckets)
    }

    /// Indexes the internal table given a Zobrist hash.
    pub fn index(&self, key: u64) -> usize {
        ((key as u128).wrapping_mul(self.buckets as u128) >> 64) as usize
    }

    /// Gets an entry from the transposition table.
    pub fn get(&self, key: u64) -> Option<&TTEntry> {
        self.table[self.index(key)].as_ref()
    }

    /// Stores an entry in the transposition table and returns its index.
    pub fn store(&mut self, key: u64, entry: TTEntry) -> usize {
        // TODO sprt different replacement strategies

        let index = self.index(key);
        self.table[index] = Some(entry);

        index
    }

    /// Clears out the transposition table.
    pub fn clear(&mut self) {
        self.table.clear();
    }
}