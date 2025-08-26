pub const BTN_LEFT: u32 = 0x110;
pub const BTN_RIGHT: u32 = 0x111;
pub const BTN_MIDDLE: u32 = 0x112;
pub const BTN_BACK: u32 = 0x113;
pub const BTN_FORWARD: u32 = 0x114;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PointerEvent {
    Motion { time: u32, dx: f64, dy: f64 },

    Button { time: u32, button: u32, state: u32 },

    Axis { time: u32, axis: u8, value: f64 },

    AxisDiscrete120 { axis: u8, value: i32 },
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum KeyboardEvent {
    Key {
        time: u32,
        key: u32,
        state: u8,
    },

    Modifiers {
        depressed: u32,
        latched: u32,
        locked: u32,
        group: u32,
    },
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Event {
    Pointer(PointerEvent),
    Keyboard(KeyboardEvent),
}
