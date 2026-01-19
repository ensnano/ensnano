use std::{collections::BTreeMap, sync::Arc};

use ahash::HashMap;
use ensnano_design::{
    curves::time_nucl_map::AbscissaConverter,
    helices::{Helices, NuclCollection},
    nucl::Nucl,
    strands::Extremity,
};
use ensnano_utils::{Referential, torsion::Torsion};
use ultraviolet::{Isometry2, Vec2, Vec3};

pub trait FlatSceneDesignReaderExt: 'static {
    fn get_all_strand_ids(&self) -> Vec<usize>;
    /// Return a the list of consecutive domain extremities of strand `s_id`. Return None iff there
    /// is no strand with id `s_id` in the design.
    fn get_strand_points(&self, s_id: usize) -> Option<Vec<Nucl>>;
    fn get_strand_color(&self, s_id: usize) -> Option<u32>;
    fn get_insertions(&self, s_id: usize) -> Option<Vec<Nucl>>;
    fn get_copy_points(&self) -> Vec<Vec<Nucl>>;
    fn get_visibility_helix(&self, h_id: usize) -> Option<bool>;
    fn get_suggestions(&self) -> Vec<(Nucl, Nucl)>;
    fn has_helix(&self, h_id: usize) -> bool;
    fn get_isometry(&self, h_id: usize, segment_idx: usize) -> Option<Isometry2>;
    fn get_helix_segment_symmetry(&self, h_id: usize, segment_idx: usize) -> Option<Vec2>;
    fn can_start_builder_at(&self, nucl: Nucl) -> bool;
    fn prime3_of_which_strand(&self, nucl: Nucl) -> Option<usize>;
    fn prime5_of_which_strand(&self, nucl: Nucl) -> Option<usize>;
    fn get_helices_map(&self) -> &Helices;
    fn is_xover_end(&self, nucl: &Nucl) -> Extremity;
    fn get_identifier_nucl(&self, nucl: &Nucl) -> Option<u32>;
    fn get_id_of_strand_containing_nucl(&self, nucl: &Nucl) -> Option<usize>;
    fn get_position_of_nucl_on_helix(
        &self,
        nucl: Nucl,
        referential: Referential,
        on_axis: bool,
    ) -> Option<Vec3>;
    fn get_torsions(&self) -> HashMap<(Nucl, Nucl), Torsion>;
    fn get_xovers_list_with_id(&self) -> Vec<(usize, (Nucl, Nucl))>;
    fn get_id_of_strand_containing_elt(&self, e_id: u32) -> Option<usize>;
    fn get_id_of_of_helix_containing_elt(&self, e_id: u32) -> Option<usize>;
    fn get_xover_with_id(&self, xover_id: usize) -> Option<(Nucl, Nucl)>;
    fn get_basis_map(&self) -> Arc<HashMap<Nucl, char>>;
    fn get_group_map(&self) -> Arc<BTreeMap<usize, bool>>;
    fn get_strand_ends(&self) -> Vec<Nucl>;
    fn get_nucl_collection(&self) -> Arc<NuclCollection>;
    fn get_abscissa_converter(&self, h_id: usize) -> AbscissaConverter;
}
