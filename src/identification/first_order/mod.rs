use aule::prelude::{Delay, Signal, Tf};
use std::{fmt::Display, time::Duration};

pub mod hagglund;
pub mod krishnaswamy;
pub mod smith;
pub mod sundaresan;
pub mod ziegler_nichols;

pub fn find_time_at_value<'a>(
    signals: impl Iterator<Item = &'a Signal<f64>>,
    value: f64,
) -> &'a Signal<f64> {
    let mut closest_signal = None;
    let mut min_diff = f64::INFINITY;

    for sig in signals {
        let diff = (sig.value - value).abs();
        if diff < min_diff {
            min_diff = diff;
            closest_signal = Some(sig);
        }
    }

    closest_signal.unwrap()
}

#[derive(Debug, Clone, PartialEq)]
pub struct FirstOrderModel {
    pub k: f64,
    pub tau: f64,
    pub theta: f64,
}

impl TryFrom<FirstOrderModel> for (Tf<f64>, Delay<f64>) {
    type Error = String;

    fn try_from(value: FirstOrderModel) -> Result<Self, Self::Error> {
        if value.theta.is_sign_negative() {
            return Err(format!("Theta cannot be negative: {}", value.theta));
        }

        let tf = Tf::new(&[value.k], &[value.tau, 1.0]);
        let delay = Delay::<f64>::new(Duration::from_secs_f64(value.theta));

        Ok((tf, delay))
    }
}

impl Display for FirstOrderModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "K: {}, θ: {}, τ: {}", self.k, self.theta, self.tau)
    }
}

pub trait FirstOrderIdentification {
    fn from_step_response(&self, signals: Vec<Signal<f64>>) -> Option<FirstOrderModel>;
}
