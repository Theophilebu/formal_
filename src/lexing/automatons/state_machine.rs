#[derive(PartialEq, Eq)]
pub enum RunInfo {
    Ready,
    Running,
    Finished,
}


pub trait StateMachine<SYMBOL, STATE>{
    fn clear(&mut self);
    fn get_run_info(& self) -> &RunInfo;
    fn is_finished(&self) -> bool;
    fn update(&mut self, symbol: &SYMBOL);
    fn get_state(&self) -> &Option<STATE>;
}