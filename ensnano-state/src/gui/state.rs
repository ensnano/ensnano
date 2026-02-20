#[derive(Debug, Clone, Copy)]
pub enum RevolutionParameterId {
    SectionParameter(usize),
    Twist, // HalfTurnCount,
    RevolutionRadius,
    NbSpiral,
    NbSectionPerSegment,
    ScaffoldLenTarget,
    SpringStiffness,
    TorsionStiffness,
    FluidFriction,
    BallMass,
    TimeSpan,
    SimulationStep,
}
