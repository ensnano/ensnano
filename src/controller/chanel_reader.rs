/*
ENSnano, a 3d graphical application for DNA nanostructures.
    Copyright (C) 2021  Nicolas Levy <nicolaspierrelevy@gmail.com> and Nicolas Schabanel <nicolas.schabanel@ens-lyon.fr>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

//! This module defines the [ChannelReader] struct which is in charge of communication with
//! computation threads that can be spawned by the progam

use std::sync::mpsc;
use std::sync::{Arc, Mutex, Weak};

use crate::app_state::{
    ShiftOptimizationResult, ShiftOptimizerReader, SimulationInterface, SimulationReader,
    SimulationUpdate,
};
#[derive(Default)]
pub struct ChannelReader {
    scaffold_shift_optimization_progress: Option<mpsc::Receiver<f32>>,
    scaffold_shift_optimization_result: Option<mpsc::Receiver<ShiftOptimizationResult>>,
    simulation_interface: Option<Weak<Mutex<dyn SimulationInterface>>>,
}

pub enum ChannelReaderUpdate {
    /// Progress has been made in the optimization of the scaffold position
    ScaffoldShiftOptimizationProgress(f32),
    /// The optimum scaffold position has been found
    ScaffoldShiftOptimizationResult(ShiftOptimizationResult),
    SimulationUpdate(Box<dyn SimulationUpdate>),
    SimulationExpired,
}

impl ChannelReader {
    pub fn get_updates(&mut self) -> Vec<ChannelReaderUpdate> {
        let mut updates = Vec::new();
        if let Some(progress) = self.get_scaffold_shift_optimization_progress() {
            updates.push(ChannelReaderUpdate::ScaffoldShiftOptimizationProgress(
                progress,
            ));
        }
        if let Some(result) = self.get_scaffold_shift_optimization_result() {
            updates.push(ChannelReaderUpdate::ScaffoldShiftOptimizationResult(result));
        }
        let mut invalidated = false;
        if let Some(interface_ptr) = self.simulation_interface.as_ref() {
            if let Some(interface) = interface_ptr.upgrade() {
                if !interface.lock().unwrap().still_valid() {
                    invalidated = true;
                    updates.push(ChannelReaderUpdate::SimulationExpired)
                }
                if let Some(new_state) = interface.lock().unwrap().get_simulation_state() {
                    updates.push(ChannelReaderUpdate::SimulationUpdate(new_state))
                }
            } else {
                invalidated = true;
            }
        }
        if invalidated {
            self.simulation_interface = None;
        }
        updates
    }

    fn get_scaffold_shift_optimization_progress(&self) -> Option<f32> {
        self.scaffold_shift_optimization_progress
            .as_ref()
            .and_then(|chanel| chanel.try_recv().ok())
    }

    fn get_scaffold_shift_optimization_result(&self) -> Option<ShiftOptimizationResult> {
        self.scaffold_shift_optimization_result
            .as_ref()
            .and_then(|chanel| chanel.try_recv().ok())
    }
}

impl ShiftOptimizerReader for ChannelReader {
    fn attach_result_chanel(&mut self, chanel: mpsc::Receiver<ShiftOptimizationResult>) {
        self.scaffold_shift_optimization_result = Some(chanel);
    }

    fn attach_progress_chanel(&mut self, chanel: mpsc::Receiver<f32>) {
        self.scaffold_shift_optimization_progress = Some(chanel);
    }
}

impl SimulationReader for ChannelReader {
    fn attach_state(&mut self, state_chanel: &std::sync::Arc<Mutex<dyn SimulationInterface>>) {
        self.simulation_interface = Some(Arc::downgrade(state_chanel));
    }
}
