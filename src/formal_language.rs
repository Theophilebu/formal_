

// --------------------------------------------

pub struct Alphabet {
    chars: Vec<char>,   // sorted, no duplicates
}

impl Alphabet {

    pub fn new(chars: &Vec<char>) -> Self {
        let mut new_chars: Vec<char> = chars.clone();
        new_chars.sort();
        new_chars.dedup();
        Alphabet { chars: new_chars }
    }

    pub fn id(&self, c: char) -> Option<usize> {
        let result: Result<usize, usize> = self.chars.binary_search(&c);
        match result {
            Ok(index) => Some(index),
            Err(_) => None,
        }
    }

    pub fn size(&self) -> usize {
        self.chars.len()
    }
}


#[derive(PartialEq, Clone, Copy)]
pub struct Symbol {
    pub id: u16,
}

impl From<u16> for Symbol {
    fn from(value: u16) -> Self {
        Self { id: value }
    }
}

// --------------------------------------------

pub struct Token {
    pub token_type: Symbol,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn next_position(&self) -> (usize, usize) {
        // returns the line and column of the token that will come after
        let mut line = self.line;
        let mut column: usize = self.column;
        for c in self.lexeme.chars() {
            if c=='\n' {
                line += 1;
                column = 0;
            }
            else {
                column += 1;
            }
        }

        (line, column)
    }
}

// --------------------------------------------

pub struct SymbolSet {
    // acts like a container of the symbols from 0 to size(not included)
    pub size: u16,
    pub representations: Vec<String>,
}

impl SymbolSet {
    pub fn size(&self) -> u16 {
        self.size
    }

    pub fn get_representation(&self, symbol: Symbol) -> &String {
        &self.representations[symbol.id as usize]
    }
}

// --------------------------------------------

pub struct CfgSymbolSet {
    terminals: SymbolSet,
    non_terminals: SymbolSet,
}

impl CfgSymbolSet {

    pub fn new(terminals: SymbolSet, non_terminals: SymbolSet) -> CfgSymbolSet {
        CfgSymbolSet {
            terminals,
            non_terminals,
        }
    }

    pub fn offset_terminals(&self) -> u16 {
        self.non_terminals.size()
    }

    fn get_terminals(&self) -> &SymbolSet {
        &self.terminals
    }

    fn get_non_terminals(&self) -> &SymbolSet {
        &self.non_terminals
    }

    fn START(&self) -> Symbol {
        Symbol { id: 1 }
    }

    fn END(&self) -> Symbol {
        Symbol { id: self.offset_terminals() + 1 }
    }

    fn ERR_NON_TERM(&self) -> Symbol{
        Symbol { id: 0 }
    }

    fn ERR_TERM(&self) -> Symbol{
        Symbol { id: self.offset_terminals() }
    }

}

// --------------------------------------------

pub struct CfgRule {
    origin: Symbol,
    replacement: Vec<Symbol>,
}

// --------------------------------------------

pub struct Cfg {
    /// augmented grammar
    symbol_set: CfgSymbolSet,
    rules: Vec<Vec<CfgRule>>,
}

impl  Cfg {

    fn new(symbol_set: CfgSymbolSet, rules: Vec<Vec<CfgRule>>) -> Cfg
    {
        Cfg { symbol_set, rules }
    }
}

// --------------------------------------------

struct CfgData<'grammar> {
    // some baked data
    cfgrammar: &'grammar Cfg,
/*
    nullable_symbols: HashSet<&'grammar Symbol>,
    
    self.augmented: bool|None = None

    self.max_replacement_length: int | None = None  #

    self.NTsymbols_implied_by_rule: dict[CFRule, set[Symbol]] | None = None  #
    self.NTsymbols_implied_by_symbol: dict[Symbol, set[Symbol]] | None = None  #

    self.nullable_symbols: dict[Symbol, bool] | None = None  #

    self.rules_directly_producing: dict[Symbol, list[CFRule]] | None = None  #
    self.rules_indirectly_producing: dict[Symbol, list[CFRule]] | None = None  #

    self.terminating_symbols: dict[Symbol, bool] | None = None  #

    self.smallest_word_indirectly_produced: dict[Symbol, int] | None = None

    self.first_sets: dict[Symbol, set[Symbol]] | None = None    #
    self.follow_sets: dict[Symbol, set[Symbol]] | None = None   #
    self.predict_sets: dict[CFRule: set[Symbol]] | None = None  #

    self.first_k_sets: dict[int, dict[Symbol, set[TupleSymbolicWord]]] = {}
    self.follow_k_sets: dict[int, dict[Symbol, set[TupleSymbolicWord]]] = {}
    self.predict_k_sets: dict[int, dict[CFRule: set[TupleSymbolicWord]]] = {}
     */
}

impl <'grammar> CfgData<'grammar> {

}