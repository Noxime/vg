use anyhow::Result;
use generational_arena::{Arena, Index};
use get_size::GetSize;
use vg_interface::WaitReason;

use crate::executor::InstanceData;

use super::executor::{DefaultExecutor, Executor, Instance};

pub struct SaveState<E: Executor = DefaultExecutor> {
    instance: E::Instance,
    saves: Arena<Save>,
}

pub struct SaveId(Index);

#[derive(GetSize)]
struct Save {
    data: InstanceData,
}

impl<E: Executor> SaveState<E> {
    pub fn new(wasm: &[u8]) -> Result<Self> {
        let instance = E::create(wasm, true)?;

        Ok(Self {
            instance,
            saves: Arena::new(),
        })
    }

    pub fn step(&mut self) -> WaitReason {
        self.instance.step()
    }

    /// Creates a save state for this instance
    pub fn save(&mut self) -> SaveId {
        SaveId(self.saves.insert(Save {
            data: self.instance.get_data(),
        }))
    }

    /// Restores a previously made save state
    pub fn restore(&mut self, id: SaveId) {
        let save = self.saves.get(id.0).expect("Unknown SaveId");
        self.instance.set_data(&save.data);
    }

    /// Get memory used for one save
    pub fn memory_size(&self, id: SaveId) -> usize {
        let save = self.saves.get(id.0).expect("Unknown SaveId");
        save.get_size()
    }

    /// Approximate memory used for all saves
    pub fn total_memory(&self) -> usize {
        self.saves.iter().map(|(_, save)| save.get_size()).sum()
    }
}
