mod dfa_lexer;
mod stack_lexer;


pub struct GeneralToken<INPUT, TOKEN_TYPE: Clone> {
    token_type: TOKEN_TYPE,
    lexeme: Vec<INPUT>,
    position: usize,
    // position of the first INPUT element(usually a char) of the token in the input stream
}

pub struct Token<SYMB: Clone> {
    TSymb: SYMB,
    lexeme: String,
    line: usize,
    cloumn: usize,
}