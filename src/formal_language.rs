use std::{cell::{OnceCell, RefCell}, rc::{Rc, Weak}};

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

// to store the index of a CfgRule inside a Vec<CfgRule> of Cfg.rules
// meaning that there can (only) be 2^8 = 256 rules for a single symbol
type CfgRuleIdBySymbol = u8;

// to store a value that represents a CfgRule but in a contiguous way
// meaning that there can (only) be 2^16 = 65578 rules in total
type CfgRuleId = u16;

// to easily get a CfgRule in a Vec<Vec<CfgRule>>
type CfgRuleCoo = (Symbol, CfgRuleIdBySymbol);


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
    rules: Vec<Vec<CfgRule>>,
    nbr_rules: CfgRuleId,
    // maps a CfgRuleFlatId to the corresponding 2-dimensionnal coordinates in rules
    rule_id_correspondance: Vec<CfgRuleCoo>,

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
        // sorts the rules to match the order of the non_terminals in the symbol_set

        let nbr_non_terminals: usize = symbol_set.get_non_terminals().size() as usize;
        let nbr_terminals: usize = symbol_set.get_terminals().size() as usize;

        let mut nbr_rules: CfgRuleId = 0;
        let mut rule_id_correspondance: Vec<CfgRuleCoo> = Vec::new();

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

                rule_id_correspondance.push((
                    Symbol { id: SymbolId::try_from(non_terminal_id).unwrap() },
                    CfgRuleIdBySymbol::try_from(cfg_rule_id_index).unwrap(),
                ));
            }
        }

        rule_id_correspondance.shrink_to_fit();

        Ok(Cfg {
            symbol_set,
            rules,
            nbr_rules,
            rule_id_correspondance,

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
        (0..(self.nbr_non_terminals() as SymbolId)).map(|id: SymbolId| Symbol {id})
    }

    pub fn all_terminals(&self) -> impl Iterator<Item = Symbol> {
        (self.nbr_non_terminals()..(self.nbr_symbols() as SymbolId)).map(|id: SymbolId| Symbol {id})
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
        // no check: we assume that every CfgRuleFlatId constructed is valid
        let (origin, index_by_origin) = self.rule_id_correspondance[id as usize];
        &self.rules[origin.id as usize][index_by_origin as usize]
    }

    // --------

    pub fn nbr_rules(&self) -> CfgRuleId {
        return self.nbr_rules;
    }

    pub fn all_rules(&self) -> impl Iterator<Item = &CfgRule> {
        self.rules.iter().flatten()
    }

    pub fn get_rules_by_origin(&self, origin: Symbol) -> &Vec<CfgRule> {
        return &self.rules[origin.id as usize];
    }
    
    /// returns an iterator which go through each (rule_id, rule) of the grammar that can produce produced_symbol
    pub fn get_rules_producing(&self, produced_symbol: Symbol) -> impl Iterator<Item = (CfgRuleId, &CfgRule)> {
        
        let rules_producing_each_symbol: &Vec<Vec<CfgRuleId>> = 
            self.rules_producing_each_symbol.get_or_init(|| self.compute_rules_producing_each_symbol());

        (&rules_producing_each_symbol[produced_symbol.id as usize])
            .iter()
            .map(|&rule_id: &CfgRuleId| (rule_id, self.get_rule_by_id(rule_id)))
    }

    fn compute_rules_producing_each_symbol(&self) -> Vec<Vec<CfgRuleId>> {
        // stores a result of type Vec<Vec<CfgRuleCoo>>
        // for each symbol, gives a list of the rules whose replacement contain the symbol

        // indexed by symbols produced
        let mut rules_producing_each_symbol: Vec<Vec<CfgRuleId>> = vec![Vec::with_capacity(10); self.nbr_symbols() as usize];

        
        // only one heap allocation(maybe one more if there is a large rule)
        let mut distinct_symbols_found: Vec<Symbol> = Vec::with_capacity(10);

        let mut rule_id: CfgRuleId = 0;

        for origin in self.all_non_terminals() {
            for (_, rule) in self.get_rules_by_origin(origin).iter().enumerate() {

                distinct_symbols_found.clear();
                
                for &replacement_symbol in &rule.replacement {

                    if distinct_symbols_found.contains(&replacement_symbol) {
                        continue;
                    }
                    rules_producing_each_symbol[replacement_symbol.id as usize].push(rule_id);
                    distinct_symbols_found.push(replacement_symbol);
                }

                rule_id += 1;
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
        for (id, rule) in self.all_rules().enumerate() {
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

