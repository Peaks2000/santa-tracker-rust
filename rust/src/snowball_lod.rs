//! Source Elf.lod/Elf.update root visibility and shared clientPlayerMarker state.
//! Copyright 2020 Google LLC, Apache-2.0. Model I/O completion is separate.
#[derive(Clone, Copy, Debug)]
pub struct ElfLod {
    pub high: bool,
    pub needs_update: bool,
    pub root_visible: bool,
    /// Source modelInitialized means requested, not that the promise resolved.
    pub model_requested: bool,
}
impl Default for ElfLod {
    fn default() -> Self {
        Self {
            high: false,
            needs_update: true,
            root_visible: true,
            model_requested: false,
        }
    }
}
impl ElfLod {
    pub fn assign(&mut self, alive: bool, high: bool) {
        if alive && self.high != high {
            self.high = high;
            self.needs_update = true;
        }
    }
    pub fn update(&mut self, alive: bool) {
        if self.needs_update {
            self.root_visible = self.high && alive;
            self.model_requested |= self.root_visible;
            self.needs_update = false;
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ClientRing {
    pub owner: Option<usize>,
    pub visible: bool,
}
impl Default for ClientRing {
    fn default() -> Self {
        Self {
            owner: None,
            visible: true,
        }
    }
}
impl ClientRing {
    pub fn setup(&mut self, player: usize) {
        self.owner = Some(player);
    }
    pub fn update(
        &mut self,
        player: usize,
        is_client: bool,
        arrived: bool,
        alive: bool,
        exiting: bool,
    ) {
        if is_client {
            self.visible = arrived;
        }
        if !alive && exiting && self.owner == Some(player) {
            self.owner = None;
        }
    }
    /// An old dolly can retain the source marker after teardown, but it is no
    /// longer in this scene. Keep mesh visibility until the next Elf update.
    pub fn leave_scene(&mut self) {
        self.owner = None;
    }
}
