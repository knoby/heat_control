machine!(
    enum HeatControl {
        Init { time: u8 },
        BufferDisabled,
        BufferEnabled,
    }
);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Tick {
    pub time: u16,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Enable {}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Disable {}

transitions!(HeatControl,
[
    (Init, Tick) => [BufferDisabled, Init],
    (BufferDisabled, Enable) => BufferEnabled,
    (BufferEnabled, Disable) => BufferDisabled
]);

impl Init {
    pub fn on_tick(self, input: Tick) -> HeatControl {
        if true {
            HeatControl::BufferDisabled(BufferDisabled {})
        } else {
            HeatControl::Init(self)
        }
    }
}

impl BufferDisabled {
    pub fn on_enable(self, _: Enable) -> BufferEnabled {
        BufferEnabled {}
    }
}

impl BufferEnabled {
    pub fn on_disable(self, _: Disable) -> BufferDisabled {
        BufferDisabled {}
    }
}
