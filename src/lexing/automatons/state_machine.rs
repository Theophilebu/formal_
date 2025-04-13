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

pub struct FiniteAutomatonState<RETURN, DATA> {
    // data might often be empty type
    pub return_value: ReturnValue<RETURN>,
    pub data: DATA,
}

pub trait StateMachine<SYMBOL, STATE>{
    // SYMBOL is what the machine reads
    fn clear(&mut self);
    fn get_run_info(& self) -> &RunInfo;
    fn update(&mut self, symbol: &SYMBOL);
    fn get_state(&self) -> &STATE;
    // if the machine is finished, get_state returns a reference to the last state the machine was in
 
    fn is_finished(&self) -> bool {
        *self.get_run_info() == RunInfo::Finished
    }
    fn is_ready(&self) -> bool {
        *self.get_run_info() == RunInfo::Ready
    }
    fn is_running(&self) -> bool {
        *self.get_run_info() == RunInfo::Running
    }
}

