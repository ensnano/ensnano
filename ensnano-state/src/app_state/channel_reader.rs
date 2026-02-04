//! This module defines the [ChannelReader] struct which is in charge of communication with
//! computation threads that can be spawned by the program

use crate::app_state::design_interactor::{
    controller::{shift_optimization::ShiftOptimizationResult, simulations::SimulationInterface},
    presenter::SimulationUpdate,
};
use std::sync::{Arc, Mutex, Weak, mpsc};

#[derive(Default, Clone)]
pub struct ScaffoldShiftReader {
    scaffold_shift_optimization_progress: Option<Arc<Mutex<mpsc::Receiver<f32>>>>,
    scaffold_shift_optimization_result: Option<Arc<Mutex<mpsc::Receiver<ShiftOptimizationResult>>>>,
}

#[derive(Default, Clone)]
pub struct SimulationInterfaceHandle {
    simulation_interface: Option<Weak<Mutex<dyn SimulationInterface>>>,
}

pub enum ChannelReaderUpdate {
    /// Progress has been made in the optimization of the scaffold position
    ScaffoldShiftOptimizationProgress(f32),
    /// The optimum scaffold position has been found
    ScaffoldShiftOptimizationResult(ShiftOptimizationResult),
}

pub enum SimulationInterfaceUpdate {
    SimulationUpdate(Box<dyn SimulationUpdate>),
    SimulationExpired,
}

impl ScaffoldShiftReader {
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
        updates
    }

    fn get_scaffold_shift_optimization_progress(&self) -> Option<f32> {
        self.scaffold_shift_optimization_progress
            .as_ref()
            .and_then(|channel| channel.lock().unwrap().try_recv().ok())
    }

    fn get_scaffold_shift_optimization_result(&self) -> Option<ShiftOptimizationResult> {
        self.scaffold_shift_optimization_result
            .as_ref()
            .and_then(|channel| channel.lock().unwrap().try_recv().ok())
    }

    pub fn attach_result_chanel(&mut self, channel: mpsc::Receiver<ShiftOptimizationResult>) {
        self.scaffold_shift_optimization_result = Some(Arc::new(Mutex::new(channel)));
    }

    pub fn attach_progress_chanel(&mut self, channel: mpsc::Receiver<f32>) {
        self.scaffold_shift_optimization_progress = Some(Arc::new(Mutex::new(channel)));
    }
}

impl SimulationInterfaceHandle {
    pub fn get_updates(&mut self) -> Vec<SimulationInterfaceUpdate> {
        let mut updates = Vec::new();
        let mut invalidated = false;
        if let Some(interface_ptr) = self.simulation_interface.as_ref() {
            if let Some(interface) = interface_ptr.upgrade() {
                if !interface.lock().unwrap().still_valid() {
                    invalidated = true;
                    updates.push(SimulationInterfaceUpdate::SimulationExpired);
                }
                if let Some(new_state) = interface.lock().unwrap().get_simulation_state() {
                    updates.push(SimulationInterfaceUpdate::SimulationUpdate(new_state));
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

    pub fn attach_state(&mut self, state_chanel: &Arc<Mutex<dyn SimulationInterface>>) {
        self.simulation_interface = Some(Arc::downgrade(state_chanel));
    }
}
