mod bk;
mod hamming;
mod utils;

impl bk::Distancable<hamming::Distance> for hamming::Position {
    fn distance(&self, other: &Self) -> hamming::Distance {
        hamming::distance(self, other)
    }
}

pub type ImagesTree = bk::Tree<hamming::Position, hamming::Distance>;
