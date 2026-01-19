use ensnano_design::curves::torus::CurveDescriptor2D;
use ultraviolet::{Rotor3, Vec3};

use crate::state::GuiAppState;

#[derive(Clone, Copy)]
pub struct RevolutionScaling {
    pub nb_helix: usize,
}

#[derive(Clone)]
pub struct CurveDescriptorBuilder<S: GuiAppState> {
    pub curve_name: &'static str,
    pub parameters: &'static [CurveDescriptorParameter],
    pub bezier_path_id: &'static (dyn Fn(&[InstantiatedParameter]) -> Option<usize> + Send + Sync),
    pub build:
        &'static (dyn Fn(&[InstantiatedParameter], &S) -> Option<CurveDescriptor2D> + Send + Sync),
    pub frame: &'static (dyn Fn(&[InstantiatedParameter], &S) -> Option<Frame> + Send + Sync),
}

impl<S: GuiAppState> std::fmt::Debug for CurveDescriptorBuilder<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("CurveDescriptorBuilder")
            .field("curve_name", &self.curve_name)
            .finish()
    }
}

impl<S: GuiAppState> std::fmt::Display for CurveDescriptorBuilder<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.curve_name)
    }
}

impl<S: GuiAppState> PartialEq for CurveDescriptorBuilder<S> {
    fn eq(&self, other: &Self) -> bool {
        self.curve_name == other.curve_name
    }
}

impl<S: GuiAppState> Eq for CurveDescriptorBuilder<S> {}

#[derive(Debug, Clone)]
pub struct CurveDescriptorParameter {
    pub name: &'static str,
    pub default_value: InstantiatedParameter,
}

pub type Frame = (Vec3, Rotor3);

#[derive(Debug, Clone, Copy)]
pub enum InstantiatedParameter {
    Float(f64),
    Int(isize),
    Uint(usize),
}

impl InstantiatedParameter {
    pub fn get_float(self) -> Option<f64> {
        if let Self::Float(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn get_int(self) -> Option<isize> {
        if let Self::Int(x) = self {
            Some(x)
        } else {
            None
        }
    }

    pub fn get_uint(self) -> Option<usize> {
        if let Self::Uint(x) = self {
            Some(x)
        } else {
            None
        }
    }
}
