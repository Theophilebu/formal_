#[derive(PartialEq, Eq)]
pub enum RunInfo {
    Ready,
    Running,
    Finished,
}

pub enum ReturnValue<RETURN>
{
    NotAccepted,
    Accepted,
    Value(RETURN),
}

pub struct State<RETURN, DATA> {
    // data might often be empty type
    return_value: ReturnValue<RETURN>,
    data: DATA,
}

pub trait StateMachine<SYMBOL, RETURN, DATA>{
    // SYMBOL is what the machine reads
    // RETURN is the output of the machine along with Accepted and NotAccepted
    // DATA is the data associated with the states
    fn clear(&mut self);
    fn get_run_info(& self) -> &RunInfo;
    fn is_finished(&self) -> bool;
    fn update(&mut self, symbol: &SYMBOL);
    fn get_state(&self) -> Option<&State<RETURN, DATA>>;
}

