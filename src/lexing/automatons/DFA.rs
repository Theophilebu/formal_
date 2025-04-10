/*
use crate::symbol::Symbol;
use super::state_machine::{RunInfo, StateMachine};
use crate::bitset::{ContiguousList, BitSet, BitSetIter};


//<'list, 'symbol, 'f, SYMBOL, F>

struct StateTransition<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
{
    origin: usize,
    target: usize,
    symbols: BitSet<'list, 'symbol, 'f, SYMBOL, F>,
    // origin is repetitive information
}

enum ReturnValue<SYMBOL>
{
    NotAccepted,
    Accepted,
    Value(SYMBOL),
}



struct DFA<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
{
    // start_state is 0
    symbols: &'list ContiguousList<'symbol, 'f, SYMBOL, F>,  // max size: 2^16
    transitions: Vec<Vec<StateTransition<'list, 'symbol, 'f, SYMBOL, F>>>,
    return_values: Vec<ReturnValue<SYMBOL>>,    // return value of each state

    // transitions[origin] = vector of outgoing transition, each with a different target
}

impl <'list, 'symbol, 'f, SYMBOL, F> DFA<'list, 'symbol, 'f, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> Option<u16>,
    'symbol: 'list,
    'symbol: 'f,
{

    fn new(symbols: &'list ContiguousList<'symbol, 'f, SYMBOL, F>,
    transitions: Vec<Vec<StateTransition<'list, 'symbol, 'f, SYMBOL, F>>>, 
    return_values: Vec<ReturnValue<SYMBOL>>,) 
    -> Result<DFA<'list, 'symbol, 'f, SYMBOL, F>, String> {

        // transitions is assumed to be indexed by origin(usize) and by 
        // checks that for each pair (state, symbol), there is at most one transition possible.
        //      returns Err varient otherise
        // symbols should be non-empty
        ////// must have at least one transition
        // symbol_id should return the index of a symbol in the slice

        if symbols.len()==0{
            return Err(String::from("this implementation of a DFA can't use an empty slice of symbols"));
        }

        if transitions.len()==0{
            return Err(String::from("this implementation of a DFA must have at least one transition"));
        }
        

        for from_origin in symbols {
            let mut outgoing_symbols: BitSet<'list, 'symbol, 'f, SYMBOL, F> 
                = BitSet::new(symbols);

            for transition in &transitions[origin_index] {
                
                if !outgoing_symbols.is_disjoint(&transition.symbols) {
                    return Err(String::from("the automaton defined by the parameters would not be deterministic"));
                }

                outgoing_symbols.update_union(&transition.symbols);
            }
        }

        
        Ok(DFA{
            symbols,
            symbol_id,
            transitions,
            return_values,
        })
    }

    fn get_transition(&self, origin: usize, symbol: &SYMBOL) -> Option<&StateTransition>{
        for outgoing_transition in self.transitions
        .get(origin)
        .expect("invalid origin")  {

            if outgoing_transition.symbols.contains((self.symbol_id)(symbol) as usize) {
                return Some(outgoing_transition);
            }
        }

        None
    }

    fn get_return_value(&self, state: usize) -> &ReturnValue {
        return &self.return_values[state];
    }

}





struct DfaRunner<'symbol, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> u16,
{
    dfa: DFA<'symbol, SYMBOL, F>,
    current_state: Option<usize>,
    run_info: RunInfo,
}


impl <'symbol, SYMBOL, F> StateMachine<SYMBOL, usize> for DfaRunner<'symbol, SYMBOL, F>
where
    SYMBOL: PartialEq,
    F: Fn(&SYMBOL) -> u16,
{
    fn clear(&mut self){
        self.current_state = Some(0);
        self.run_info = RunInfo::Ready;
    }

    fn get_run_info(&self) -> &RunInfo {
        &self.run_info
    }

    fn get_state(&self) -> &Option<usize> {
        &self.current_state
    }

    fn get_return_value(&self) -> &ReturnValue {
        self.dfa.get_return_value(self.current_state)
    }

    fn is_finished(&self) -> bool {
        self.run_info == RunInfo::Finished
    }

    fn update(&mut self, symbol: &SYMBOL) {
        
        if self.is_finished(){
            panic!();
        }

        let Some(current_state) = self.current_state else{
            panic!();
        };

        let transition = self.dfa.get_transition(current_state, symbol);
        match transition{
            None => {
                self.run_info = RunInfo::Finished;
                self.current_state = None;
            }
            Some(actual_transition) => {
                self.run_info = RunInfo::Running;
                self.current_state = Some(actual_transition.target);
            } 
        }
    }
}

*/