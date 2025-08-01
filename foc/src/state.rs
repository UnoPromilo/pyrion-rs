pub struct Uninitialized;
pub struct Disabled;

pub struct Motor<State> {
    state: State,
}

impl<State> Motor<State> {
    pub fn new() -> Self {
        Self {
            state: Uninitialized,
        }
    }
}

impl Motor<Uninitialized> {
    pub fn initialize(self) -> Motor<Uninitialized> {

    }


}
