#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(all(unix, not(target_os = "macos")))]
mod wayland;
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) use wayland::*;

use std::collections::{HashMap, HashSet};

pub use common::*;
pub use eyre::Result;

pub(crate) enum EmulationKind {
    #[cfg(target_os = "macos")]
    MacOS(MacOSEmulation),
}

impl EmulationKind {
    pub async fn consume(
        &mut self,
        event: Event,
        #[allow(unused_variables)] handle: u32,
    ) -> Result<()> {
        match self {
            #[cfg(target_os = "macos")]
            EmulationKind::MacOS(emulation) => emulation.consume(event).await,
            #[cfg(all(unix, not(target_os = "macos")))]
            EmulationKind::Wayland(emulation) => capture.create(pos).await,
        }
    }

    pub async fn create(&mut self, #[allow(unused_variables)] handle: u32) {
        match self {
            #[cfg(target_os = "macos")]
            EmulationKind::MacOS(_) => {}
            #[cfg(all(unix, not(target_os = "macos")))]
            EmulationKind::Wayland(emulation) => capture.create(pos).await,
        }
    }
}

pub struct Emulation {
    emulation: EmulationKind,
    #[allow(dead_code)]
    handles: HashSet<u32>,
    pressed_keys: HashMap<u32, HashSet<u32>>,
}

impl Emulation {
    pub fn new() -> Result<Self> {
        Ok(Self {
            emulation: {
                #[cfg(target_os = "macos")]
                let emulation = EmulationKind::MacOS(MacOSEmulation::new()?);

                #[cfg(all(unix, not(target_os = "macos")))]
                let capture = CaptureKind::Wayland(LayerShellInputCapture::new()?);

                emulation
            },
            handles: Default::default(),
            pressed_keys: Default::default(),
        })
    }

    fn update_pressed_keys(&mut self, handle: u32, key: u32, state: u8) -> bool {
        let Some(pressed_keys) = self.pressed_keys.get_mut(&handle) else {
            return false;
        };

        if state == 0 {
            pressed_keys.remove(&key)
        } else {
            pressed_keys.insert(key)
        }
    }

    pub async fn create(&mut self, handle: u32) -> bool {
        if self.handles.insert(handle) {
            self.pressed_keys.insert(handle, HashSet::new());
            self.emulation.create(handle).await;
            true
        } else {
            false
        }
    }

    pub async fn consume(&mut self, event: Event, handle: u32) -> Result<()> {
        match event {
            Event::Keyboard(KeyboardEvent::Key { key, state, .. }) => {
                // prevent double pressed / released keys
                if self.update_pressed_keys(handle, key, state) {
                    self.emulation.consume(event, handle).await?;
                }
                Ok(())
            }
            _ => self.emulation.consume(event, handle).await,
        }
    }
}
