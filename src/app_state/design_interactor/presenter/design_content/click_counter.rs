#[derive(Clone, Debug)]
pub(super) struct ClickCounter {
    value: u32,
}

impl ClickCounter {
    pub(super) fn new() -> Self {
        Self { value: 0 }
    }

    pub(super) fn set(&mut self, value: u32) {
        self.value = value;
    }

    pub(super) fn inc(&mut self) -> u32 {
        let ret = self.value;
        self.value += 1;
        ret
    }
}
