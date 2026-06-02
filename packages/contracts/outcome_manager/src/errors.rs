use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum OutcomeError {
    AlreadyInitialized = 1,
    InvalidQuorum = 2,
    UnauthorizedOracle = 3,
    AlreadySettled = 4,
    DuplicateSubmission = 5,
    InvalidOutcome = 6,
    CallNotSettled = 7,
    AlreadyClaimed = 8,
    NothingToClaim = 9,
    InvalidWinningStake = 10,
    Overflow = 11,
    CallNotFinalized = 12,
    InvalidFeeBps = 13,
    ContractPaused = 14,
    MaxOraclesReached = 15,
    SubmissionWindowExpired = 16,
    EmptyBatch = 17,
    LengthMismatch = 18,
    NotInitialized = 19,
    FeeCollectorNotSet = 20,
    ObservationOutOfOrder = 21,
    InsufficientPriceObservations = 22,
    NoPriceObservations = 23,
    ZeroTimeWindow = 24,
    RegistryNotSet = 25,
    DisputeWindowExpired = 26,
}
