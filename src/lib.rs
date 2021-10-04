pub mod config;
pub mod data;
mod metrics;
mod queries;
pub mod tcp;
mod utils;

impl queries::Distancable<metrics::hamming::Distance> for metrics::hamming::Position {
    fn distance(&self, other: &Self) -> metrics::hamming::Distance {
        metrics::hamming::distance(self, other)
    }
}
