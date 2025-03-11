use sea_orm::prelude::Decimal;

pub fn take_commission(amount: Decimal, commission: Decimal) -> CommissionData {
    let commission = (amount * commission).round_dp(3);
    CommissionData {
        commission,
        amount: (amount - commission).round_dp(3),
    }
}

pub struct CommissionData {
    pub commission: Decimal,
    pub amount: Decimal
}