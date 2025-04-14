use std::marker::PhantomData;

use crate::lexing::automatons::{dfa::*, state_machine::*};
use crate::datastructures::{table1d::Table1D, table2d::Table2D, option_uint::OptionUint};
use crate::lexing::lexers::{GeneralToken, Token};

type SINT = i16;

pub struct DfaLexer<TOKEN_TYPE: Clone, TABLE, STATES>
where
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<TOKEN_TYPE, ()>>,
{
    dfa: Dfa<SINT, TABLE, TOKEN_TYPE, (), STATES>,
}

pub struct DfaLexerRunner<'dfa, TOKEN_TYPE: Clone, TABLE, STATES>
where
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<TOKEN_TYPE, ()>>,
{
    dfa_runner: DfaRunner<'dfa, SINT, TABLE, TOKEN_TYPE, (), STATES>,
    error_output: TOKEN_TYPE,

    current_state: Vec<GeneralToken<usize, TOKEN_TYPE>>,
    current_lexeme: Vec<usize>,
    lexeme_position: usize,
    position: usize,
    run_info: RunInfo,
}



impl <'dfa, TOKEN_TYPE: Clone, TABLE, STATES> StateMachine<usize, Vec<GeneralToken<usize, TOKEN_TYPE>>> 
for DfaLexerRunner<'dfa, TOKEN_TYPE, TABLE, STATES>
where
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<TOKEN_TYPE, ()>>,
{
    fn clear(&mut self) {
        self.current_state = vec![];
    }

    fn get_run_info(& self) -> &RunInfo {
        &self.run_info
    }

    fn update(&mut self, symbol: &usize) {

        self.handle_dfa_end();
        // restarts the dfa_runner if needed, with additionnal steps 

        self.dfa_runner.update(symbol);
        self.position+=1;
        self.current_lexeme.push(*symbol);

    }

    fn get_state(&self) -> &Vec<GeneralToken<usize, TOKEN_TYPE>> {
        &self.current_state
    }

}


impl <'dfa, TOKEN_TYPE: Clone, TABLE, STATES> DfaLexerRunner<'dfa, TOKEN_TYPE, TABLE, STATES>
    where
    TABLE: Table2D<OptionUint<SINT>>,
    STATES: Table1D<FiniteAutomatonState<TOKEN_TYPE, ()>>,
{

    fn handle_dfa_end(&mut self) {
        if self.dfa_runner.is_finished() {
            let last_symbol: usize = self.current_lexeme.pop().unwrap();

            let final_state_id: usize = *self.dfa_runner.get_state();
            let final_state_return_value: &ReturnValue<TOKEN_TYPE> = 
                &self.dfa_runner
                    .get_dfa()
                    .get_state(final_state_id)
                    .return_value;

            let token_type_found: TOKEN_TYPE = match final_state_return_value {
                ReturnValue::Accepted => self.error_output.clone(),
                ReturnValue::NotAccepted => self.error_output.clone(),
                ReturnValue::Value(output_value) => output_value.clone(),
            };

            self.current_state.push(GeneralToken{
                token_type: token_type_found,
                lexeme: self.current_lexeme.clone(),
                position: self.lexeme_position,
            });

            self.dfa_runner.clear();
            self.dfa_runner.update(&last_symbol);

            if self.dfa_runner.is_finished() {
                // the symbol that can't be at the end of the last lexeme 
                // also can't be at the start of a new lexeme
                // we add an error token with this symbol only as the lexeme

                self.current_state.push(GeneralToken{
                    token_type: self.error_output.clone(),
                    lexeme: vec![last_symbol],
                    position: self.position - 1,
                });

                self.dfa_runner.clear();
                self.lexeme_position = self.position;
                self.current_lexeme = vec![];
            }
            else {
                self.lexeme_position = self.position - 1;
                self.current_lexeme = vec![last_symbol];
            }

        }
    }

    pub fn finish(&mut self) {
        self.handle_dfa_end();

        if self.current_lexeme.len() != 0 {

            let final_state_id: usize = *self.dfa_runner.get_state();
            let final_state_return_value: &ReturnValue<TOKEN_TYPE> = 
                &self.dfa_runner
                    .get_dfa()
                    .get_state(final_state_id)
                    .return_value;

            let token_type_found: TOKEN_TYPE = match final_state_return_value {
                ReturnValue::Accepted => self.error_output.clone(),
                ReturnValue::NotAccepted => self.error_output.clone(),
                ReturnValue::Value(output_value) => output_value.clone(),
            };

            self.current_state.push(GeneralToken{
                token_type: token_type_found,
                lexeme: self.current_lexeme.clone(),
                position: self.lexeme_position,
            });
        }

        
    }
}


