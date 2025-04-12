use std::marker::PhantomData;

use num::{Signed, PrimInt};

use std::num::NonZeroUsize;
use thiserror::Error;

use super::state_machine::{RunInfo, State, StateMachine};
use crate::datastructures::{option_uint::OptionUint, table1d::Table1D, table2d::Table2D, bitset::*};
/*
#[error("IO error: {0}")]
    Io(#[from] std::io::Error),
*/

type UINT = u8; // used for Bitsets, might slightly affect performance and memory usage?


#[derive(Error, Debug)]
pub enum NfaError {
    #[error("Position (i={i} , j={j}) is out of bounds")]
    IndexOutOfBounds{i: usize, j: usize},

    // #[error("State {state} has multiple transitions on {symb}, it would introduce nondeterminism")]
    // NotDeterministic{state: usize, symb: usize},
}




struct NfaStateTransition {
    origin_state_id: usize,
    symbol_read_id: usize,
    target_states_id: BitSet<UINT>,
}



struct Nfa<SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<State<RETURN, DATA>>
{
    // start_state is 0
    // SINT is a signed integer, including i8, .., i128
    // i8 can handle 128 states, i16 can handle 32768 states, i_n can handle 2^(n-1) states 
    nbr_symbols: NonZeroUsize,
    nbr_states: NonZeroUsize,
    transition_table: TABLE,
    states: STATES,
    phantom: PhantomData<(SINT, RETURN, DATA)>,

    // transition_table.get(origin_state_id, symbol_read_id) = target_state_id
}

/*
impl <SINT, TABLE, RETURN, DATA, STATES> Dfa<SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<State<RETURN, DATA>>
{
    fn from_table(table: TABLE, states: STATES) -> Self {
        Dfa {
            nbr_symbols: table.width(),
            nbr_states: table.height(),
            transition_table: table,
            states,
            phantom: PhantomData,
        }
    }

    fn from_transitions(nbr_symbols: NonZeroUsize, nbr_states: NonZeroUsize, transitions: Vec<StateTransition>, states: STATES) 
    -> Result<Dfa<SINT, Vec<Vec<OptionUint<SINT>>>, RETURN, DATA, STATES>, DfaError> {

        // checks that for each pair (state, symbol), there is at most one transition possible.
        //      returns None variant otherwise

        let mut table: Vec<Vec<OptionUint<SINT>>> = 
        vec![vec![OptionUint::from(None);nbr_symbols.into()]; nbr_states.into()];

        for transition in transitions {
            let current_value: Option<usize> = 
            Table2D::get(&table, transition.origin_state_id, transition.symbol_read_id).get_value();

            if let None = current_value{
                return Err(DfaError::NotDeterministic {
                    state: transition.origin_state_id,
                    symb: transition.symbol_read_id,
                });
            }

            let new_value: Option<usize> = Some(transition.target_state_id);
            table[transition.origin_state_id][transition.symbol_read_id] = OptionUint::from(new_value);
        }
        
        Ok(Dfa{
            nbr_symbols,
            nbr_states,
            transition_table: table,
            states,
            phantom: PhantomData,
        })
    }

    fn next_state_id(&self,current_state_id: usize ,symbol_read_id: usize) -> Option<usize> {
        self.transition_table.get(current_state_id, symbol_read_id).get_value()
    }

    fn get_state(&self, state_id: usize) -> &State<RETURN, DATA> {
        &self.states.get(state_id)
    }
}





struct DfaRunner<'dfa, SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<State<RETURN, DATA>>
{
    dfa: &'dfa Dfa<SINT, TABLE, RETURN, DATA, STATES>,
    current_state_id: Option<usize>,
    run_info: RunInfo,
}


impl <'dfa, SINT, TABLE, RETURN, DATA, STATES> StateMachine<usize, RETURN, DATA> 
for DfaRunner<'dfa, SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<State<RETURN, DATA>>
{


/*
fn clear(&mut self);
    fn get_run_info(& self) -> &RunInfo;
    fn is_finished(&self) -> bool;
    fn update(&mut self, symbol: &SYMBOL);
    fn get_state(&self) -> Option<&State<RETURN, DATA>>;
}
*/

    fn clear(&mut self){
        self.current_state_id = Some(0);
        self.run_info = RunInfo::Ready;
    }

    fn get_run_info(&self) -> &RunInfo {
        &self.run_info
    }
    
    fn is_finished(&self) -> bool {
        self.run_info == RunInfo::Finished
    }

    fn update(&mut self, symbol: &usize) {
        
        if self.is_finished(){
            panic!();
        }

        let Some(current_state_id) = self.current_state_id else{
            panic!();
        };

        let next_state_id: Option<usize> = self.dfa.next_state_id(current_state_id, *symbol);
        match next_state_id {
            None => {
                self.run_info = RunInfo::Finished;
                self.current_state_id = None;
            }
            Some(actual_next_state_id) => {
                self.run_info = RunInfo::Running;
                self.current_state_id = Some(actual_next_state_id);
            } 
        }
    }

    fn get_state(&self) -> Option<&State<RETURN, DATA>> {
        match self.current_state_id {
            None => None,
            Some(state_id) => Some(self.dfa.get_state(state_id)),
        }
    }
}

*/