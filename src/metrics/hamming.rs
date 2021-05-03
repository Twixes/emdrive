use std::convert::TryFrom;

pub type Position = u128;
pub type Distance = u8;

const M1: Position = 0x55555555555555555555555555555555; // 01010101...
const M2: Position = 0x33333333333333333333333333333333; // 00110011...
const M4: Position = 0x0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f; // 00001111...
const M8: Position = 0x00ff00ff00ff00ff00ff00ff00ff00ff; // and so on
const M16: Position = 0x0000ffff0000ffff0000ffff0000ffff;
const M32: Position = 0x00000000ffffffff00000000ffffffff;
const M64: Position = 0x0000000000000000ffffffffffffffff;

pub fn weight(x: Position) -> Distance {
    let mut x = (x & M1) + ((x >> 1) & M1);
    x = (x & M2) + ((x >> 2) & M2);
    x = (x & M4) + ((x >> 4) & M4);
    x = (x & M8) + ((x >> 8) & M8);
    x = (x & M16) + ((x >> 16) & M16);
    x = (x & M32) + ((x >> 32) & M32);
    x = (x & M64) + ((x >> 64) & M64);
    Distance::try_from(x).unwrap()
}

pub fn distance(x: &Position, y: &Position) -> Distance {
    weight(x ^ y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hamming_weight_is_calculated_correctly() {
        assert_eq!(weight(0x0000000000000000ffffffffffffffff), 64);
        assert_eq!(weight(0b1101), 3);
    }

    #[test]
    fn hamming_distance_is_calculated_correctly() {
        assert_eq!(distance(&0b0011, &0b0101), 2);
        assert_eq!(distance(&0b1111, &0b1111), 0);
        assert_eq!(
            distance(
                &0xf000000000000000ffffffffffffffff,
                &0x0000000000000000fff0ffffffffffff
            ),
            8
        );
        assert_eq!(distance(&0, &0xffffffffffffffffffffffffffffffff), 128);
    }
}
