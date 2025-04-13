use std::marker::PhantomData;

use num::{Signed, PrimInt};

use std::num::NonZeroUsize;
use thiserror::Error;

use super::state_machine::{RunInfo, FiniteAutomatonState, StateMachine};
use super::dfa::StateTransition;
use crate::datastructures::{option_uint::OptionUint, table1d::Table1D, table2d::Table2D, bitset::BitSet};
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




struct StateTransitionSet<TABLE>
where
    TABLE: Table1D<UINT>,
{
    origin_state_id: usize,
    symbol_read_id: usize,
    target_states_id: BitSet<UINT, TABLE>,
}



struct Nfa<SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE1D: Table1D<UINT>,
    TABLE2D: Table2D<BitSet<UINT, TABLE1D>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    // start_state is 0
    // SINT is a signed integer, including i8, .., i128
    // i8 can handle 128 states, i16 can handle 32768 states, i_n can handle 2^(n-1) states 
    nbr_symbols: NonZeroUsize,
    nbr_states: NonZeroUsize,
    transition_table: TABLE2D,
    states: STATES,
    phantom: PhantomData<(SINT, RETURN, DATA, TABLE1D)>,

    // transition_table.get(origin_state_id, symbol_read_id) = target_state_id
}

impl <SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES> Nfa<SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE1D: Table1D<UINT>,
    TABLE2D: Table2D<BitSet<UINT, TABLE1D>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    fn from_table(table: TABLE2D, states: STATES) -> Self {
        Nfa {
            nbr_symbols: table.width(),
            nbr_states: table.height(),
            transition_table: table,
            states,
            phantom: PhantomData,
        }
    }

    fn from_transitions(nbr_symbols: NonZeroUsize, nbr_states: NonZeroUsize,
        transitions: Vec<StateTransition>, states: STATES) 
    -> Nfa<SINT, Vec<Vec<BitSet<UINT, TABLE1D>>>, TABLE1D, RETURN, DATA, STATES> {
        // duplicate transitions are merged
        let mut table: Vec<Vec<BitSet<UINT, TABLE1D>>> = 
        vec![vec![BitSet::new_filled(false, nbr_states.into());nbr_symbols.into()]; nbr_states.into()];

        for transition in transitions {
            let current_value: &mut BitSet<UINT, TABLE1D> = 
            Table2D::get_mut(&mut table, transition.origin_state_id, transition.symbol_read_id);

            current_value.insert(transition.target_state_id);
        }
        
        Nfa{
            nbr_symbols,
            nbr_states,
            transition_table: table,
            states,
            phantom: PhantomData,
        }
    }

    fn from_transition_sets(nbr_symbols: NonZeroUsize, nbr_states: NonZeroUsize, 
        transition_sets: Vec<StateTransitionSet<TABLE1D>>, states: STATES) 
    -> Nfa<SINT, Vec<Vec<BitSet<UINT, TABLE1D>>>, TABLE1D, RETURN, DATA, STATES> {
        // duplicate transitions are merged
        let mut table: Vec<Vec<BitSet<UINT, TABLE1D>>> = 
        vec![vec![BitSet::new_filled(false, nbr_states.into());nbr_symbols.into()]; nbr_states.into()];

        for transition_set in transition_sets {
            let current_value: &mut BitSet<UINT, TABLE1D> = 
            Table2D::get_mut(&mut table, transition_set.origin_state_id, transition_set.symbol_read_id);

            current_value.update_union(&transition_set.target_states_id);
        }
        
        Nfa{
            nbr_symbols,
            nbr_states,
            transition_table: table,
            states,
            phantom: PhantomData,
        }
    }

    fn next_state_ids(&self, current_possible_state_ids: &BitSet<UINT, TABLE1D>, symbol_read_id: usize) -> BitSet<UINT, TABLE1D> {

        let mut next_state_ids: BitSet<UINT, TABLE1D> = BitSet::new_filled(false, self.nbr_states.into());
        
        for current_possible_state_id in current_possible_state_ids {
            next_state_ids.update_union(self.transition_table.get(current_possible_state_id, symbol_read_id));
        }

        next_state_ids
        // self.transition_table.get(current_state_id, symbol_read_id).get_value()
    }

    fn get_state(&self, state_id: usize) -> &FiniteAutomatonState<RETURN, DATA> {
        &self.states.get(state_id)
    }
}




struct NfaRunner<'nfa, SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE1D: Table1D<UINT>,
    TABLE2D: Table2D<BitSet<UINT, TABLE1D>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    nfa: &'nfa Nfa<SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES>,
    current_state_ids: BitSet<UINT, TABLE1D>,
    run_info: RunInfo,
}


impl <'nfa, SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES> StateMachine<usize, BitSet<UINT, TABLE1D>> 
for NfaRunner<'nfa, SINT, TABLE2D, TABLE1D, RETURN, DATA, STATES>
where
    SINT: Signed + PrimInt,
    TABLE1D: Table1D<UINT>,
    TABLE2D: Table2D<BitSet<UINT, TABLE1D>>,
    STATES: Table1D<FiniteAutomatonState<RETURN, DATA>>
{
    fn clear(&mut self){
        self.current_state_ids = BitSet::new_filled(false, self.nfa.nbr_states.into());
        self.current_state_ids.insert(0);
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

        // if self.current_state_ids.len()==0 {
        //     panic!();
        // };
        // logically shouldn't happen

        let next_state_id: BitSet<UINT, TABLE1D> = self.nfa.next_state_ids(&self.current_state_ids, *symbol);
        
        if next_state_id.len()==0 {
            self.run_info = RunInfo::Finished;
        }
        else {
            self.current_state_ids = next_state_id;
            self.run_info = RunInfo::Running;
        }
        
        
    }

    fn get_state(&self) -> &BitSet<UINT, TABLE1D> {
        &self.current_state_ids
    }

}
