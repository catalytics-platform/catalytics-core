#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressionEventType {
    BetaApplicantCreated = 1,
    CaticsBalanceCheck = 2,
    MineSeason0 = 3,
    CatLevelUp = 4,
    JupStaked = 5,
    ReferralCreated = 6,
}

impl ProgressionEventType {
    pub fn id(&self) -> i32 {
        *self as i32
    }
}
