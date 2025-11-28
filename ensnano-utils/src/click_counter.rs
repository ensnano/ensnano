#[derive(Clone, Debug)]
pub struct ClickCounter {
    value: u32,
}

impl ClickCounter {
    pub fn new() -> Self {
        Self { value: 0 }
    }

    pub fn set(&mut self, value: u32) {
        self.value = value;
    }

    pub fn next(&mut self) -> u32 {
        let ret = self.value;
        self.value += 1;
        ret
    }
}
