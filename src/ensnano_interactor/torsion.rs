use super::*;
type Xover = (Nucl, Nucl);
/// Represent the torsion applied on each helices implied in a cross_over.
///
/// The strength is defined as the cross-over's component in the radial acceleration of the helix
pub struct Torsion {
    /// The strength applied on the 5' helix of the cross over
    pub strength_prime5: f32,
    /// The strength applied on the 3' helix of the cross over
    pub strength_prime3: f32,
    /// Two cross-overs are fiends if their extremities are neighbor. In that case only one of
    /// of them should appear in the keys of the torsion map, and their strength are combined
    pub friend: Option<Xover>,
}
