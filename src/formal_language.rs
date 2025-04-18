use std::{cell::{OnceCell, RefCell}, rc::{Rc, Weak}};

use thiserror::Error;

use crate::datastructures::bitset::BitSet;
use crate::UINT;

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


// these values are totally arbitrary
// here, they are both just enough and largely enough 
// to handle the grammar of general-purpose programming language 

// to store the symbol efficiently
// meaning that there can (only) be 2^16 = 65578 symbols for one cfg
type SymbolId = u16;

// to store the index of a CfgRule inside a Vec<CfgRule> of Cfg.rules
// meaning that there can (only) be 2^8 = 256 rules for a single symbol
type CfgRuleIdIndex = u8;

// to store a value that represents a CfgRule but in a contiguous way
// meaning that there can (only) be 2^16 = 65578 rules in total
type CfgRuleFlatId = u16;


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
    size: SymbolId,
    representations: Vec<String>,
}

impl SymbolSet {

    pub fn new(size: SymbolId, representations: Vec<String>) -> Self {
        Self { size, representations }
    }

    pub fn size(&self) -> SymbolId {
        self.size
    }

    pub fn get_representation(&self, symbol: Symbol) -> &String {
        &self.representations[symbol.id as usize]
    }
}


// --------------------------------------------


#[derive(Error, Debug)]
pub enum CfgError {
    #[error("CfgRule  {rule:?}, has an invalid origin.")]
    InvalidRuleOrigin{rule: CfgRule},

    #[error("CfgRule  {rule:?}, has an invalid replacement symbol {symbol:?}.")]
    InvalidRuleReplacement{rule: CfgRule, symbol: Symbol},

    #[error("State Transition {transition:?} has the origin state id {} which is not a valid state id", transition.origin_state_id)]
    InvalidTransitionOrigin{transition: StateTransition},

    #[error("State Transition {transition:?} has the target state id {} which is not a valid state id", transition.target_state_id)]
    InvalidTransitionTarget{transition: StateTransition},

    #[error("The number of states({nbr_states}) is too large (max={})", SINT::MAX)]
    TooManyStates{nbr_states: usize},

    #[error("The number of states {table_height} in the table doesn't match the length({vec_len}) of the vector states")]
    WrongNbrStates{table_height: usize, vec_len: usize},

    #[error("The number of states {table_width} in the table doesn't match the size({alphabet_size}) of the alphabet")]
    WrongNbrChars{table_width: usize, alphabet_size: usize},

    #[error("The number of states and valid chars can't be 0")]
    EmptyTable,

    #[error("The table passed should be rectangular")]
    NonRectTable,
    
    #[error("The char {c} is not in the alphabet")]
    InvalidChar{c: char},

    #[error("The state id {state_id} is not valid")]
    InvalidStateId{state_id: usize},
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


#[derive(Debug, Clone, Copy)]
struct  CfgRuleId {
    symbol: Symbol,
    index: CfgRuleIdIndex,
}

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
}

// --------------------------------------------

/// augmented context-free grammar
pub struct Cfg {
    symbol_set: CfgSymbolSet,
    // indexed by non_terminals
    rules: Vec<Vec<CfgRule>>,
    nbr_rules: CfgRuleFlatId,
    // maps a CfgRuleFlatId to the corresponding CfgRuleId
    rule_flat_id_correspondance: Vec<CfgRuleId>,

    // cached values

    // indexed by symbols
    rules_producing_each_symbol: OnceCell<Vec<Vec<CfgRuleId>>>,

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

    pub fn new(symbol_set: CfgSymbolSet, rules: Vec<Vec<CfgRule>>) -> Result<Cfg, CfgError> {

        let nbr_non_terminals: usize = symbol_set.get_non_terminals().size() as usize;
        let nbr_terminals: usize = symbol_set.get_terminals().size() as usize;

        let mut nbr_rules: CfgRuleFlatId = 0;
        let mut rule_flat_id_correspondance: Vec<CfgRuleId> = Vec::new();

        // checks that each rule is valid
        for non_terminal_id in 0..nbr_non_terminals {
            for (cfg_rule_id_index, rule) in (&rules[non_terminal_id]).iter().enumerate() {
                if rule.origin.id as usize >= nbr_non_terminals {
                    return Err(CfgError::InvalidRuleOrigin { rule: rule.clone() });
                }

                for replacement_symbol in &rule.replacement {
                    if replacement_symbol.id as usize>= nbr_non_terminals + nbr_terminals {
                        return Err(CfgError::InvalidRuleReplacement { rule: rule.clone(), symbol: *replacement_symbol });
                    }
                }

                nbr_rules += 1; 

                rule_flat_id_correspondance.push(CfgRuleId {
                    symbol: Symbol { id: SymbolId::try_from(non_terminal_id).unwrap() },
                    index: CfgRuleIdIndex::try_from(cfg_rule_id_index).unwrap(),
                });
            }
        }



        Ok(Cfg {
            symbol_set,
            rules,
            nbr_rules,
            rule_flat_id_correspondance,

            rules_producing_each_symbol: OnceCell::new(),
            are_symbols_nullable: OnceCell::new(),

        })
    }

    // -------------------------- utility methods

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
        (0..(self.nbr_non_terminals() as SymbolId)).map(|id: SymbolId| Symbol {id})
    }

    pub fn all_terminals(&self) -> impl Iterator<Item = Symbol> {
        (0..(self.nbr_terminals() as SymbolId)).map(|id: SymbolId| Symbol {id})
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = Symbol> {
        (0..((self.nbr_terminals() + self.nbr_non_terminals()) as SymbolId)).map(|id: SymbolId| Symbol {id})
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
        &self.rules[id.symbol.id as usize][id.index as usize]
    }

    fn get_rule_by_flat_id(&self, flat_id: CfgRuleFlatId) -> &CfgRule {
        // no check: we assume that every CfgRuleFlatId constructed is valid
        return self.get_rule_by_id(self.rule_flat_id_correspondance[flat_id as usize]);
    }

    // --------

    pub fn nbr_rules(&self) -> CfgRuleFlatId {
        return self.nbr_rules;
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &CfgRule> {
        self.rules.iter().flatten()
    }

    pub fn all_rules_with_ids(&self) -> impl Iterator<Item = (CfgRuleFlatId, CfgRuleId, &CfgRule)> {
        self.rule_flat_id_correspondance
            .iter()
            .enumerate()
            .map(|(rule_flat_id, &rule_id)|
                (CfgRuleFlatId::try_from(rule_flat_id).unwrap(), rule_id, self.get_rule_by_id(rule_id)))
    }

    pub fn get_rules_by_origin(&self, origin: Symbol) -> &Vec<CfgRule> {
        return &self.rules[origin.id as usize];
    }
    
    /// returns an iterator which go through each rule of the grammar that can produce produced_symbol
    pub fn get_rules_producing(&self, produced_symbol: Symbol) -> impl Iterator<Item = &CfgRule> {
        
        let rules_producing_each_symbol: &Vec<Vec<CfgRuleId>> = 
            self.rules_producing_each_symbol.get_or_init(|| self.compute_rules_producing_each_symbol());

        (&rules_producing_each_symbol[produced_symbol.id as usize])
            .iter()
            .map(|rule_id: &CfgRuleId| self.get_rule_by_id(*rule_id))
    }

    fn compute_rules_producing_each_symbol(&self) -> Vec<Vec<CfgRuleId>> {
        // stores a result of type Vec<Vec<CfgRuleId>>
        // for each symbol, gives a list of the rules whose replacement contain the symbol

        // indexed by symbols produced
        let mut rules_producing_each_symbol: Vec<Vec<CfgRuleId>> = vec![Vec::with_capacity(10); self.nbr_symbols() as usize];

        
        // let distinct_symbols_found: SmallVec<[Symbol; 10]> = SmallVec::new(); ?

        // only one heap allocation(maybe one more if there is a large rule)
        let mut distinct_symbols_found: Vec<Symbol> = Vec::with_capacity(10);

        for origin in self.all_non_terminals() {
            for (index, rule) in self.get_rules_by_origin(origin).iter().enumerate() {

                distinct_symbols_found.clear();
                
                for &replacement_symbol in &rule.replacement {

                    if distinct_symbols_found.contains(&replacement_symbol) {
                        continue;
                    }
                    let rule_id: CfgRuleId = CfgRuleId { symbol: origin, index: index.try_into().unwrap()};
                    rules_producing_each_symbol[replacement_symbol.id as usize].push(rule_id);
                    distinct_symbols_found.push(replacement_symbol);
                }
            }
        }

        for symbol_produced in self.all_symbols() {
            rules_producing_each_symbol[symbol_produced.id as usize].shrink_to_fit();
        }

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
        for (flat_id, _, rule) in self.all_rules_with_ids() {
            if rule.is_empty(){
                if !are_nullable.contains(rule.origin.id as usize) {
                    unprocessed_nullable_symbols.push(rule.origin);
                }
                are_nullable.insert(rule.origin.id as usize);
                nbr_nullable[flat_id as usize] = 0
            }
            else {
                let (term, non_term, dist_non_term) = self.count_symbols_in_rule(rule);
                // if all the symbols in rule.replacement are NTsymbols, it could be nullable
                // In the opposite case, we won't even consider the rule because we know it is not nullable
                if term == 0 {
                    nbr_nullable[flat_id as usize] = dist_non_term;
                }
            }
        }

        while unprocessed_nullable_symbols.len() > 0 {
            let unprocessed_nullable_symbol: Symbol = unprocessed_nullable_symbols.pop().unwrap();
            for rule in self.get_rules_producing(unprocessed_nullable_symbol) {
                self.get
                nbr_nullable[rule] -= 1;

                if nbr_nullable[rule] == 0 and not is_nullable[rule.origin] {
                    is_nullable[rule.origin] = True
                    stack_collapsed.append(rule.origin)
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


