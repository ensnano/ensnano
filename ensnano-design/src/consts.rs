#[derive(strum::Display)]
pub enum IterativeFrameAlgorithm {
    BasedOnGeometry, // impose by the file format
}

pub const ITERATIVE_AXIS_ALGORITHM: IterativeFrameAlgorithm =
    // either use original iterative frame algorithm
    // IterativeFrameAlgorithm::Original;
    // or use tangent-rotation-based frame algorithm
    IterativeFrameAlgorithm::BasedOnGeometry;
