#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub(crate) use macos::*;

#[cfg(all(unix, not(target_os = "macos")))]
mod wayland;
#[cfg(all(unix, not(target_os = "macos")))]
pub(crate) use wayland::*;

use futures::{Stream, ready};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::{
    mem::swap,
    task::{Context, Poll},
};

pub use eyre::Result;
pub use futures::StreamExt;
pub use okbm_common::*;

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum CaptureEvent {
    Begin,

    Input(Event),
}

#[derive(Debug, Clone, Copy, Eq, Hash, PartialEq)]
pub enum Position {
    Left,
    Right,
    Top,
    Bottom,
}

impl Position {
    pub fn opposite(&self) -> Self {
        match self {
            Position::Left => Self::Right,
            Position::Right => Self::Left,
            Position::Top => Self::Bottom,
            Position::Bottom => Self::Top,
        }
    }
}

pub enum CaptureKind {
    #[cfg(target_os = "macos")]
    MacOS(MacOSInputCapture),
    #[cfg(all(unix, not(target_os = "macos")))]
    Wayland(LayerShellInputCapture),
}

impl CaptureKind {
    pub fn poll_next_unpin(
        &mut self,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<(Position, CaptureEvent)>>> {
        match self {
            #[cfg(target_os = "macos")]
            CaptureKind::MacOS(capture) => capture.poll_next_unpin(cx),
            #[cfg(all(unix, not(target_os = "macos")))]
            CaptureKind::Wayland(capture) => capture.poll_next_unpin(cx),
        }
    }

    pub async fn create(&mut self, pos: Position) -> Result<()> {
        match self {
            #[cfg(target_os = "macos")]
            CaptureKind::MacOS(capture) => capture.create(pos).await,
            #[cfg(all(unix, not(target_os = "macos")))]
            CaptureKind::Wayland(capture) => capture.create(pos).await,
        }
    }

    pub async fn release(&mut self) -> Result<()> {
        match self {
            #[cfg(target_os = "macos")]
            CaptureKind::MacOS(capture) => capture.release().await,
            #[cfg(all(unix, not(target_os = "macos")))]
            CaptureKind::Wayland(capture) => capture.release().await,
        }
    }
}

pub struct Capture {
    capture: CaptureKind,

    pressed_keys: HashSet<scancode::Linux>,

    position_map: HashMap<Position, Vec<u32>>,

    id_map: HashMap<u32, Position>,

    pending: VecDeque<(u32, CaptureEvent)>,
}

impl Capture {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            capture: {
                #[cfg(target_os = "macos")]
                let capture = CaptureKind::MacOS(MacOSInputCapture::new().await?);

                #[cfg(all(unix, not(target_os = "macos")))]
                let capture = CaptureKind::Wayland(LayerShellInputCapture::new()?);

                capture
            },
            pressed_keys: Default::default(),
            position_map: Default::default(),
            id_map: Default::default(),
            pending: Default::default(),
        })
    }

    pub async fn create(&mut self, id: u32, pos: Position) -> Result<()> {
        assert!(!self.id_map.contains_key(&id));

        self.id_map.insert(id, pos);

        if let Some(v) = self.position_map.get_mut(&pos) {
            v.push(id);
            Ok(())
        } else {
            self.position_map.insert(pos, vec![id]);
            self.capture.create(pos).await
        }
    }

    pub async fn release(&mut self) -> Result<()> {
        self.pressed_keys.clear();
        self.capture.release().await
    }

    fn update_pressed_keys(&mut self, key: u32, state: u8) {
        if let Ok(scancode) = scancode::Linux::try_from(key) {
            println!("key: {key}, state: {state}, scancode: {scancode:?}");

            match state {
                1 => self.pressed_keys.insert(scancode),
                _ => self.pressed_keys.remove(&scancode),
            };
        }
    }
}

impl Stream for Capture {
    type Item = Result<(u32, CaptureEvent)>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(e) = self.pending.pop_front() {
            return Poll::Ready(Some(Ok(e)));
        }

        // ready
        let event = ready!(self.capture.poll_next_unpin(cx));

        // stream closed
        let event = match event {
            Some(e) => e,
            None => return Poll::Ready(None),
        };

        // error occurred
        let (pos, event) = match event {
            Ok(e) => e,
            Err(e) => return Poll::Ready(Some(Err(e))),
        };

        // handle key presses
        if let CaptureEvent::Input(Event::Keyboard(KeyboardEvent::Key { key, state, .. })) = event {
            self.update_pressed_keys(key, state);
        }

        let len = self
            .position_map
            .get(&pos)
            .map(|ids| ids.len())
            .unwrap_or(0);

        match len {
            0 => Poll::Pending,
            1 => Poll::Ready(Some(Ok((
                self.position_map.get(&pos).expect("no id")[0],
                event,
            )))),
            _ => {
                let mut position_map = HashMap::new();
                swap(&mut self.position_map, &mut position_map);
                {
                    for &id in position_map.get(&pos).expect("position") {
                        self.pending.push_back((id, event));
                    }
                }
                swap(&mut self.position_map, &mut position_map);

                Poll::Ready(Some(Ok(self.pending.pop_front().expect("event"))))
            }
        }
    }
}
