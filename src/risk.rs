use crate::types::Side;

pub fn allow(side: &Side, pos: i64) -> bool {
    match side {
        Side::Buy => pos < 5,
        Side::Sell => pos > -5,
        Side::Hold => true,
    }
}
