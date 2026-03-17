use crate::{
    identification::first_order::{FirstOrderIdentification, FirstOrderModel},
    line_equation::LineEquation,
};
use aule::prelude::Signal;

pub struct Hagglund;

impl FirstOrderIdentification for Hagglund {
    fn from_step_response(&self, signals: Vec<Signal<f64>>) -> Option<FirstOrderModel> {
        if signals.len() < 3 {
            return None;
        }

        let y0 = signals.first().unwrap().value;
        let yf = signals.last().unwrap().value;
        let y632 = yf * 0.632 + y0;

        let line_eq = LineEquation::from_signals_with_maximum_slope(signals.into_iter());

        let t1 = line_eq.time_at(y0) as f64;
        let t2 = line_eq.time_at(y632) as f64;
        let theta = t1;
        let tau = t2 - t1;
        let k = yf - y0;

        Some(FirstOrderModel { k, tau, theta })
    }
}
