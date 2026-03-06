#[derive(Debug, Clone, Copy)]
pub enum RevolutionParameterId {
    SectionParameter(usize),
    Twist, // HalfTurnCount,
    RevolutionRadius,
    // NbHelices,
    // NbSpiral,
    NbSectionPerSegment,
    ScaffoldLenTarget,
    SpringStiffness,
    TorsionStiffness,
    FluidFriction,
    BallMass,
    TimeSpan,
    SimulationStep,
}
