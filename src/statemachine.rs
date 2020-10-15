machine!(
    enum HeatControl {
        Init { time: u32 },
        BufferDisabled,
        BufferEnabled,
        PumpActive { time: u32 },
        PumpPause { time: u32 },
    }
);

impl HeatControl {
    pub fn to_string(&self) -> &'static str {
        match self {
            HeatControl::Error => "Error",
            HeatControl::Init(_) => "Init",
            HeatControl::BufferDisabled(_) => "Buffer Disabled",
            HeatControl::BufferEnabled(_) => "Buffer Enabled",
            HeatControl::PumpActive(_) => "Pump Active",
            HeatControl::PumpPause(_) => "Pump Pause",
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            HeatControl::Error => 0,
            HeatControl::Init(_) => 1,
            HeatControl::BufferDisabled(_) => 2,
            HeatControl::BufferEnabled(_) => 3,
            HeatControl::PumpActive(_) => 4,
            HeatControl::PumpPause(_) => 5,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Tick {
    pub time: u32,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Enable {}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Disable {}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ActivatePump {
    pub time: u32,
}

transitions!(HeatControl,
[
    (Init, Tick) => [BufferDisabled, Init],
    (BufferDisabled, Enable) => BufferEnabled,
    (BufferEnabled, Disable) => BufferDisabled,
    (BufferEnabled, ActivatePump) => PumpActive,
    (PumpActive, Tick) => [PumpActive, PumpPause],
    (PumpPause, Tick) => [PumpPause, BufferEnabled]
]);

impl Init {
    pub fn on_tick(self, input: Tick) -> HeatControl {
        if input.time.wrapping_sub(self.time) > 5_000 {
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

    pub fn on_activate_pump(self, input: ActivatePump) -> PumpActive {
        PumpActive { time: input.time }
    }
}

impl PumpActive {
    pub fn on_tick(self, input: Tick) -> HeatControl {
        if input.time.wrapping_sub(self.time) > 60_000 {
            HeatControl::PumpPause(PumpPause { time: input.time })
        } else {
            HeatControl::PumpActive(self)
        }
    }
}

impl PumpPause {
    pub fn on_tick(self, input: Tick) -> HeatControl {
        if input.time.wrapping_sub(self.time) > 60_000 {
            HeatControl::BufferEnabled(BufferEnabled {})
        } else {
            HeatControl::PumpPause(self)
        }
    }
}
