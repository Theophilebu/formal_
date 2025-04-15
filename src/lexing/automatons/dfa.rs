use std::marker::PhantomData;

use num::{Signed, PrimInt};

use std::num::NonZeroUsize;
use thiserror::Error;

use super::state_machine::{RunInfo, FiniteAutomatonState, StateMachine};
use crate::datastructures::{option_uint::OptionUint, table1d::Table1D, table2d::Table2D};
/*
#[error("IO error: {0}")]
    Io(#[from] std::io::Error),
*/



#[derive(Error, Debug)]
pub enum DfaError {
    #[error("Position (i={i} , j={j}) is out of bounds")]
    IndexOutOfBounds{i: usize, j: usize},

    #[error("State {state} has multiple transitions on {symb}, it would introduce nondeterminism")]
    NotDeterministic{state: usize, symb: usize},
}




#[derive(Debug, Clone)]
pub struct StateTransition {
    pub origin_state_id: usize,
    pub symbol_read_id: usize,
    pub target_state_id: usize,
}



pub struct Dfa<SINT, TABLE, RETURN: Clone, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
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


impl <SINT, TABLE, RETURN: Clone, DATA, STATES> Dfa<SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    pub fn from_table(table: TABLE, states: STATES) -> Self {
        Dfa {
            nbr_symbols: table.width(),
            nbr_states: table.height(),
            transition_table: table,
            states,
            phantom: PhantomData,
        }
    }

    pub fn from_transitions(nbr_symbols: NonZeroUsize, nbr_states: NonZeroUsize, transitions: Vec<StateTransition>, states: STATES) 
    -> Result<Dfa<SINT, Vec<Vec<OptionUint<SINT>>>, RETURN, DATA, STATES>, DfaError> {

        // checks that for each pair (state, symbol), there is at most one transition possible.
        //      returns None variant otherwise

        let mut table: Vec<Vec<OptionUint<SINT>>> = 
        vec![vec![OptionUint::from(None);nbr_symbols.into()]; nbr_states.into()];

        for transition in transitions {
            let current_value: Option<usize> = 
            Table2D::get(&table, transition.origin_state_id, transition.symbol_read_id).get_value();

            if let Some(_) = current_value {
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

    pub fn next_state_id(&self, current_state_id: usize ,symbol_read_id: usize) -> Option<usize> {
        self.transition_table.get(current_state_id, symbol_read_id).get_value()
    }

    pub fn get_state(&self, state_id: usize) -> &FiniteAutomatonState<RETURN, DATA> {
        &self.states.get(state_id)
    }
}





pub struct DfaRunner<'dfa, SINT, TABLE, RETURN: Clone, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    dfa: &'dfa Dfa<SINT, TABLE, RETURN, DATA, STATES>,
    current_state_id: usize,
    run_info: RunInfo,
}

impl <'dfa, SINT, TABLE, RETURN: Clone, DATA, STATES> DfaRunner<'dfa, SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    pub fn new(dfa: &'dfa Dfa<SINT, TABLE, RETURN, DATA, STATES>) -> Self {
        DfaRunner {
            dfa: dfa,
            current_state_id: 0,
            run_info: RunInfo::Ready,
        }
    }

    pub fn get_dfa(&self) -> &'dfa Dfa<SINT, TABLE, RETURN, DATA, STATES> {
        self.dfa
    }
}

impl <'dfa, SINT, TABLE, RETURN: Clone, DATA, STATES> StateMachine<usize, usize> 
for DfaRunner<'dfa, SINT, TABLE, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    fn clear(&mut self){
        self.current_state_id = 0;
        self.run_info = RunInfo::Ready;
    }

    fn get_run_info(&self) -> &RunInfo {
        &self.run_info
    }

    fn update(&mut self, symbol: &usize) {
        
        if self.is_finished(){
            panic!();
        }

        let next_state_id: Option<usize> = self.dfa.next_state_id(self.current_state_id, *symbol);
        match next_state_id {
            None => {
                self.run_info = RunInfo::Finished;
            }
            Some(actual_next_state_id) => {
                self.run_info = RunInfo::Running;
                self.current_state_id = actual_next_state_id;
            } 
        }
    }

    fn get_state(&self) -> &usize {
        &self.current_state_id
    }

}

