#[cfg(feature = "ensnano_upcoming")]
use crate::curves::CurveConstructor;
#[cfg(feature = "ensnano_upcoming")]
use crate::{curves::circle_curve::CircleCurve, parameters::HelixParameters};
#[cfg(feature = "ensnano_upcoming")]
use ensnano_upcoming::{EllipticTorusConcentricCircleDescriptor, TorusConcentricCircleDescriptor};

#[cfg(feature = "ensnano_upcoming")]
impl CurveConstructor for TorusConcentricCircleDescriptor {
    type Curve = CircleCurve;

    fn instantiate_with_parameters(&self, parameters: HelixParameters) -> Self::Curve {
        let (radius, z, perimeter, abscissa_converter_factor, target_nb_nt, is_closed) = self
            .instantiate(
                parameters.inter_helix_axis_gap() as f64,
                parameters.rise as f64,
                HelixParameters::GEARY_2014_DNA.rise as f64,
            );

        CircleCurve {
            radius,
            z,
            perimeter,
            abscissa_converter_factor,
            target_nb_nt,
            is_closed,
        }
    }
}

#[cfg(feature = "ensnano_upcoming")]
impl CurveConstructor for EllipticTorusConcentricCircleDescriptor {
    type Curve = CircleCurve;

    fn instantiate_with_parameters(&self, parameters: HelixParameters) -> Self::Curve {
        let (radius, z, perimeter, abscissa_converter_factor, target_nb_nt, is_closed) = self
            .instantiate(
                parameters.rise as f64,
                HelixParameters::GEARY_2014_DNA.rise as f64,
            );

        CircleCurve {
            radius,
            z,
            perimeter,
            abscissa_converter_factor,
            target_nb_nt,
            is_closed,
        }
    }
}
