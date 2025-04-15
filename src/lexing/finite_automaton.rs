mod dfa;
mod nfa;


pub enum ReturnValue<RETURN: Clone>
{
    NotAccepted,
    Accepted,
    Value(RETURN),
}


pub struct FiniteAutomatonState<RETURN: Clone, DATA> {
    // data might often be empty type
    pub return_value: ReturnValue<RETURN>,
    pub data: DATA,
}