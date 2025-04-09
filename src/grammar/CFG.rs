use crate::symbol::{self, Symbol};
use std::collections::HashSet;


fn test(){

    let terminals: Vec<String> = vec![String::from("a"), String::from("b"), String::from("c")];
    let non_terminals: Vec<String> = vec![String::from("A"), String::from("B"), String::from("C")];

    let alphabet: CFG_Alphabet = CFG_Alphabet::new(
        terminals, 
        non_terminals
    );


    let (a, b, c) = (
        &(alphabet.get_terminals()[0]),
        &(alphabet.get_terminals()[1]),
        &(alphabet.get_terminals()[2]),
    );
    let (A, B, C) = (
        &(alphabet.get_non_terminals()[0]),
        &(alphabet.get_non_terminals()[1]),
        &(alphabet.get_non_terminals()[2]),
    );

    let rule: CFG_Rule = CFG_Rule { origin: A, replacement: vec![A, b, c, B] };

    let G: CFG = CFG::new(&alphabet, vec![rule]);

}

struct CFG_Alphabet{
    symbols: Vec<Symbol>,
    representations: Vec<String>,
    index_begin_terminals: usize,
}


impl <'alp> CFG_Alphabet{
    
    fn new(terminals: Vec<String>, non_terminals: Vec<String>) -> CFG_Alphabet{
        // ERR_NON_TERM, START, non_terminals, ERR_TERM, END, terminals
        // contract: less than 2^16 - 4 symbols (65532: should be fine)
        // takes ownership of the inputs
        let total_size: usize = 4+terminals.len()+non_terminals.len();
        let mut symbols: Vec<Symbol> = Vec::new();
        symbols.reserve_exact(total_size);

        let mut representations: Vec<String> = Vec::new();
        representations.reserve_exact(total_size);


        let cap: usize = symbols.capacity();
        println!("{cap}");

        let mut current_id: u16 = 0;

        symbols.push(Symbol { id: current_id });
        representations.push(String::from("ERR_NON_TERM"));
        current_id += 1;

        symbols.push(Symbol { id: current_id });
        representations.push(String::from("START"));
        current_id += 1;

        for non_term_repr in non_terminals{
            symbols.push(Symbol { id: current_id });
            representations.push(non_term_repr);
            current_id += 1;
        }

        let index_begin_terminals: usize = usize::from(current_id);

        for non_term_repr in terminals{
            symbols.push(Symbol { id: current_id });
            representations.push(non_term_repr);
            current_id += 1;
        } 

        symbols.push(Symbol { id: current_id });
        representations.push(String::from("ERR_TERM"));
        current_id += 1;

        symbols.push(Symbol { id: current_id });
        representations.push(String::from("END"));
        current_id += 1;

        CFG_Alphabet{
            symbols,
            representations,
            index_begin_terminals
        }


    }

    fn get_terminals(&self) -> &[Symbol] {
        &self.symbols[self.index_begin_terminals..]
    }

    fn get_non_terminals(&self) -> &[Symbol] {
        &self.symbols[0..self.index_begin_terminals]
    }

    fn START(&'alp self) -> &'alp Symbol{
        &self.symbols[1]
    }

    fn END(&'alp self) -> &'alp Symbol{
        &self.symbols[(self.index_begin_terminals+1) as usize]
    }

    fn ERR_NON_TERM(&'alp self) -> &'alp Symbol{
        &self.symbols[0]
    }

    fn ERR_TERM(&'alp self) -> &'alp Symbol{
        &self.symbols[(self.index_begin_terminals) as usize]
    }

}

struct CFG_Rule<'alp>{
    origin: &'alp Symbol,
    replacement: Vec<&'alp Symbol>,
}


struct CFG<'alp>{
    /// augmented grammar
    alphabet: &'alp CFG_Alphabet,
    rules: Vec<CFG_Rule<'alp>>,
}

struct CFG_Data<'grammar>{
    cfgrammar: &'grammar CFG<'grammar>,

    nullable_symbols: HashSet<&'grammar Symbol>,
    /*
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


impl <'gram, 'alp> CFG<'gram>{

    fn new(alphabet: &'alp CFG_Alphabet, rules: Vec<CFG_Rule<'alp>>) -> CFG<'gram>
    where 'alp:'gram
    {
        CFG{alphabet, rules}
    }

    fn START(&'gram self) -> &'gram Symbol {
        self.alphabet.START()
    }

    fn END(&'gram self) -> &'gram Symbol{
        self.alphabet.END()
    }

    fn ERR_NON_TERM(&'gram self) -> &'gram Symbol{
        self.alphabet.ERR_NON_TERM()
    }

    fn ERR_TERM(&'gram self) -> &'gram Symbol{
        self.alphabet.ERR_TERM()
    }
}
