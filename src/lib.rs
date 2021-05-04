mod utils;
mod metrics;
mod queries;
mod data;
pub mod server;
pub mod config;

impl queries::Distancable<metrics::hamming::Distance> for metrics::hamming::Position {
    fn distance(&self, other: &Self) -> metrics::hamming::Distance {
        metrics::hamming::distance(self, other)
    }
}

pub type ImagesTree = queries::bk::Tree<metrics::hamming::Position, metrics::hamming::Distance>;
