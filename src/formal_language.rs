use std::cell::OnceCell;
use crate::datastructures::flat_table::FlatTable;

use thiserror::Error;

use crate::datastructures::bitset::BitSet;
use crate::UINT;

// --------------------------------------------

pub struct Alphabet {
    chars: Vec<char>,   // sorted, no duplicates
}

impl Alphabet {

    pub fn new(mut chars: Vec<char>) -> Self {
        chars.sort();
        chars.dedup();
        Alphabet { chars: chars }
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


// these values are totally arbitrary
// here, they are both just enough and largely enough 
// to handle the grammar of general-purpose programming language 

// to store the symbol efficiently
// meaning that there can (only) be 2^16 = 65578 symbols for one cfg
type SymbolId = u16;


// to store a value that represents a CfgRule but in a contiguous way
// meaning that there can (only) be 2^16 = 65578 rules in total
type CfgRuleId = u16;


const EXPECTED_RULE_SIZE: usize = 10;

#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Symbol {
    pub id: SymbolId,
}

impl From<SymbolId> for Symbol {
    fn from(value: SymbolId) -> Self {
        Self { id: value }
    }
}

// --------------------------------------------

#[derive(Debug)]
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
    representations: Vec<String>,
}

impl SymbolSet {

    pub fn new(representations: Vec<String>) -> Self {
        Self { representations }
    }

    pub fn size(&self) -> SymbolId {
        self.representations.len().try_into().unwrap()
    }

    pub fn get_representation(&self, local_id: SymbolId) -> &String {
        &self.representations[local_id as usize]
    }

    // debug only
    pub fn get_id(&self, representation: &str) -> Option<SymbolId> {
        self.representations
            .iter()
            .position(|s| s==representation)
            .map(|x| SymbolId::try_from(x).unwrap())

    }
}




// --------------------------------------------


#[derive(Error, Debug)]
pub enum CfgError {
    #[error("CfgRule  {rule:?}, has an invalid origin.")]
    InvalidRuleOrigin{rule: CfgRule},

    #[error("CfgRule  {rule:?}, has an invalid replacement symbol {symbol:?}.")]
    InvalidRuleReplacement{rule: CfgRule, symbol: Symbol},
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

    pub fn offset_terminals(&self) -> SymbolId {
        self.non_terminals.size()
    }

    pub fn get_terminals(&self) -> &SymbolSet {
        &self.terminals
    }

    pub fn get_non_terminals(&self) -> &SymbolSet {
        &self.non_terminals
    }

    pub fn START(&self) -> Symbol {
        Symbol { id: 1 }
    }

    pub fn END(&self) -> Symbol {
        Symbol { id: self.offset_terminals() + 1 }
    }

    pub fn ERR_NON_TERM(&self) -> Symbol{
        Symbol { id: 0 }
    }

    pub fn ERR_TERM(&self) -> Symbol{
        Symbol { id: self.offset_terminals() }
    }

    // debug only
    pub fn get_symbol_by_representation(&self, representation: &str) -> Symbol {

        match representation {
            "START" => self.START(),
            "END" => self.END(),
            "ERR_TERM" => self.ERR_TERM(),
            "ERR_NON_TERM" => self.ERR_NON_TERM(),
            other => {
                Symbol {id: 
                    match self.get_non_terminals().get_id(other) {
                        Some(non_terminal_id) => non_terminal_id,
                        None => self.get_terminals().get_id(other).unwrap() + self.offset_terminals(),
                    }
                }
            }
        }
    }
}

// --------------------------------------------

#[derive(Debug, Clone)]
pub struct CfgRule {
    pub origin: Symbol,
    pub replacement: Vec<Symbol>,
}

impl CfgRule {
    /// returns true iff the replacement is empty
    pub fn is_empty(&self) -> bool {
        self.replacement.len()==0
    }

    pub fn replacement_size(&self) -> usize {
        self.replacement.len()
    }
}

// --------------------------------------------

/// augmented context-free grammar
pub struct Cfg {
    symbol_set: CfgSymbolSet,
    // indexed by non_terminals
    rules: FlatTable<CfgRule, CfgRuleId>,

    // -------------- cached values

    // indexed by produced symbols
    rules_producing_each_symbol: OnceCell<FlatTable<CfgRuleId, SymbolId>>,

    // indexed by symbols
    are_symbols_nullable: OnceCell<BitSet<UINT>>,

    /*
    get_NTsymbols_implied_by_rule
    get_NTsymbols_implied_by_symbol
    get_rules_indirectly_producing
    get_terminating_symbols
    is_word_nullable
    is_rule_nullable
    compute_first_sets
    compute_follow_sets
    compute_predict_sets
     */
}

impl  Cfg {

    pub fn new(symbol_set: CfgSymbolSet, mut rules: Vec<CfgRule>) -> Result<Cfg, CfgError> {
        // sorts the rules to match the order of the non_terminals in the symbol_set

        let nbr_non_terminals: SymbolId = symbol_set.get_non_terminals().size();
        let nbr_terminals: SymbolId = symbol_set.get_terminals().size();

        rules.sort_by(|rule1, rule2| rule1.origin.id.cmp(&rule2.origin.id));

        // checks that each rule is valid
        for (rule_id, rule) in rules.iter().enumerate() {
            if rule.origin.id >= nbr_non_terminals {
                return Err(CfgError::InvalidRuleOrigin { rule: rule.clone() });
            }

            for replacement_symbol in &rule.replacement {
                if replacement_symbol.id >= nbr_non_terminals + nbr_terminals {
                    return Err(CfgError::InvalidRuleReplacement { rule: rule.clone(), symbol: *replacement_symbol });
                }
            }
        }

        let mut rule_origin_correspondance: Vec<CfgRuleId> = Vec::with_capacity(nbr_non_terminals as usize);
        let mut current_rule_id: CfgRuleId = 0;
        for non_terminal_id in 0..nbr_non_terminals {
            rule_origin_correspondance.push(current_rule_id);
            let mut i = 0;
            while ((current_rule_id + i) as usize != rules.len()) 
                && (rules[(current_rule_id + i) as usize].origin.id == non_terminal_id) {
                i += 1;
            }
            current_rule_id += i;
        }

        Ok(Cfg {
            symbol_set,
            rules: FlatTable::new(rules, rule_origin_correspondance),

            rules_producing_each_symbol: OnceCell::new(),

            are_symbols_nullable: OnceCell::new(),

        })
    }

    // -------------------------- utility methods

    pub fn get_symbol_set(&self) -> &CfgSymbolSet {
        &self.symbol_set
    }

    pub fn nbr_non_terminals(&self) -> SymbolId {
        self.symbol_set.get_non_terminals().size()
    }

    pub fn nbr_terminals(&self) -> SymbolId {
        self.symbol_set.get_terminals().size()
    }

    pub fn nbr_symbols(&self) -> SymbolId {
        self.nbr_non_terminals() + self.nbr_terminals()
    }

    pub fn all_non_terminals(&self) -> impl Iterator<Item = Symbol> {
        (0..self.nbr_non_terminals()).map(|id: SymbolId| Symbol {id})
    }

    pub fn all_terminals(&self) -> impl Iterator<Item = Symbol> {
        (self.nbr_non_terminals()..self.nbr_symbols()).map(|id: SymbolId| Symbol {id})
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = Symbol> {
        (0..self.nbr_terminals() + self.nbr_non_terminals()).map(|id: SymbolId| Symbol {id})
    }

    pub fn is_terminal(&self, symbol: Symbol) -> bool {
        return symbol.id >= self.nbr_non_terminals()
    }

    pub fn is_non_terminal(&self, symbol: Symbol) -> bool {
        return symbol.id < self.nbr_non_terminals()
    }
    
    // --------

    /// returns the number of terminals, non_terminals, distinct non_terminals
    pub fn count_symbols_in_rule(&self, rule: &CfgRule) -> (usize, usize, SymbolId) {
        let (mut term, mut non_term, mut dist_non_term): (usize, usize, SymbolId) = (0, 0, 0);

        let mut distinct_non_terminals_found: Vec<Symbol> = Vec::with_capacity(EXPECTED_RULE_SIZE);

        for &symbol in &rule.replacement {
            if self.is_terminal(symbol) {
                term += 1;
            }
            else {
                non_term += 1;
                if !distinct_non_terminals_found.contains(&symbol) {
                    dist_non_term += 1;
                    distinct_non_terminals_found.push(symbol);
                }
            }
        }

        (term, non_term, dist_non_term)
    }

    // --------

    fn get_rule_by_id(&self, id: CfgRuleId) -> &CfgRule {
        // no check: we assume that every CfgRuleId constructed is valid
        self.rules.get_by_id(id)
    }

    // --------

    pub fn nbr_rules(&self) -> CfgRuleId {
        return self.rules.size();
    }

    pub fn all_rules(&self) -> impl Iterator<Item = (CfgRuleId, &CfgRule)> {
        self.rules
            .table
            .iter()
            .enumerate()
            .map(|(id, rule)| (CfgRuleId::try_from(id).unwrap(), rule))
    }

    pub fn get_rules_by_origin(&self, origin: Symbol) -> impl Iterator<Item = (CfgRuleId, &CfgRule)> {
        let rule_id: CfgRuleId = self.rules.rows[origin.id as usize];
        (&self.rules[origin.id])
            .iter()
            .enumerate()
            .map(move |(id, rule)| (CfgRuleId::try_from(id).unwrap() + rule_id, rule))
    }
    
    /// returns an iterator which go through each (rule_id, rule) of the grammar that can produce produced_symbol
    pub fn get_rules_producing(&self, produced_symbol: Symbol) -> impl Iterator<Item = (CfgRuleId, &CfgRule)> {
        
        let rules_producing_each_symbol: &FlatTable<CfgRuleId, SymbolId> = 
            self.rules_producing_each_symbol.get_or_init(|| self.compute_rules_producing_each_symbol());

        (&rules_producing_each_symbol[produced_symbol.id as usize])
            .iter()
            .map(|&rule_id: &CfgRuleId| (rule_id, self.get_rule_by_id(rule_id)))
    }

    fn compute_rules_producing_each_symbol(&self) -> FlatTable<CfgRuleId, SymbolId> {
        // stores a result of type Vec<Vec<CfgRuleCoo>>
        // for each symbol, gives a list of the rules whose replacement contain the symbol

        // indexed by symbols produced
        let mut rules_producing_each_symbol: Vec<Vec<CfgRuleId>> = vec![Vec::with_capacity(10); self.nbr_symbols() as usize];

        let mut flat_table_size: usize = 0;
        
        // only one heap allocation(maybe one more if there is a large rule)
        let mut distinct_symbols_found: Vec<Symbol> = Vec::with_capacity(EXPECTED_RULE_SIZE);
        for (rule_id, rule) in self.all_rules() {

            distinct_symbols_found.clear();
            
            for &replacement_symbol in &rule.replacement {

                if distinct_symbols_found.contains(&replacement_symbol) {
                    continue;
                }
                rules_producing_each_symbol[replacement_symbol.id as usize].push(rule_id);
                distinct_symbols_found.push(replacement_symbol);
                flat_table_size += 1;
            }
        }

        let mut table: Vec<CfgRuleId> = Vec::with_capacity(flat_table_size);
        let mut rows: Vec<SymbolId> = Vec::with_capacity(self.nbr_non_terminals() as usize);

        for symbol_produced in self.all_symbols() {
            rules_producing_each_symbol[symbol_produced.id as usize].shrink_to_fit();
        }

        FlatTable::new(table, rows)
        rules_producing_each_symbol
    }

    // -------------------------- nullable symbols/words/rules

    pub fn is_symbol_nullable(&self, symbol: Symbol) -> bool {
        if self.is_terminal(symbol) {
            false
        }
        else {
            self.are_symbols_nullable
                .get_or_init(|| self.compute_are_symbols_nullable())
                .contains(symbol.id as usize)
        }
    }

    pub fn is_word_nullable(&self, word: &[Symbol]) -> bool {
        word.iter().all(|&symbol| self.is_symbol_nullable(symbol))
    }

    pub fn is_rule_nullable(&self, rule_id: CfgRuleId) -> bool {
        self.get_rule_by_id(rule_id).replacement.iter().all(|&symbol| self.is_symbol_nullable(symbol))
    }


    fn compute_are_symbols_nullable(&self) -> BitSet<UINT> {
        // adapted from here:
        // https://cstheory.stackexchange.com/questions/2479/quickly-finding-empty-string-producing-nonterminals-in-a-cfg


        // initially, all symbols are considered non-nullable
        let mut are_nullable: BitSet<UINT> = BitSet::new_filled(false, self.nbr_non_terminals() as usize);

        // number of distinct non_terminals marked as non-nullable that the index rule can produce
        // will be initialised later
        let mut nbr_nullable: Vec<SymbolId> = Vec::with_capacity(self.nbr_rules() as usize);

        // stack of non_terminals that have been marked nullable but not yet processed
        // capacity is just a random guess
        let mut unprocessed_nullable_symbols: Vec<Symbol> = Vec::with_capacity((self.nbr_non_terminals()/2) as usize); 

        // initialize are_nullable and nbr_nullable and unprocessed_nullable_symbols
        for (id, rule) in self.all_rules() {
            if rule.is_empty(){
                if !are_nullable.contains(rule.origin.id as usize) {
                    unprocessed_nullable_symbols.push(rule.origin);
                }
                are_nullable.insert(rule.origin.id as usize);
                nbr_nullable[id as usize] = 0
            }
            else {
                let (term, _, dist_non_term) = self.count_symbols_in_rule(rule);
                // if all the symbols in rule.replacement are NTsymbols, it could be nullable
                // In the opposite case, we won't even consider the rule because we know it is not nullable
                if term == 0 {
                    nbr_nullable[id as usize] = dist_non_term;
                }
            }
        }

        while unprocessed_nullable_symbols.len() > 0 {
            let unprocessed_nullable_symbol: Symbol = unprocessed_nullable_symbols.pop().unwrap();
            for (id, rule) in self.get_rules_producing(unprocessed_nullable_symbol) {
                nbr_nullable[id as usize] -= 1;

                if nbr_nullable[id as usize] == 0 && !are_nullable.contains(rule.origin.id as usize) {
                    are_nullable.insert(rule.origin.id as usize);
                    unprocessed_nullable_symbols.push(rule.origin);
                }
            }
        }

        are_nullable
    }

}



// --------------------------------------------


    // self.rules_indirectly_producing: dict[Symbol, list[CFRule]] | None = None  #
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

