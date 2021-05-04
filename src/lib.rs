mod utils;
mod metrics;
mod queries;
pub mod server;
pub mod config;

impl queries::traits::Distancable<metrics::hamming::Distance> for metrics::hamming::Position {
    fn distance(&self, other: &Self) -> metrics::hamming::Distance {
        metrics::hamming::distance(self, other)
    }
}

pub type ImagesTree = queries::bk::Tree<metrics::hamming::Position, metrics::hamming::Distance>;
