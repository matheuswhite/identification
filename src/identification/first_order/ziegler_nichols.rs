use crate::{
    identification::first_order::{FirstOrderIdentification, FirstOrderModel},
    line_equation::LineEquation,
};
use aule::prelude::Signal;

pub struct ZieglerNichols;

impl FirstOrderIdentification for ZieglerNichols {
    fn from_step_response(&self, signals: Vec<Signal<f64>>) -> Option<FirstOrderModel> {
        if signals.len() < 3 {
            return None;
        }

        let y0 = signals.first().unwrap().value;
        let yf = signals.last().unwrap().value;

        let line_eq = LineEquation::from_signals_with_maximum_slope(signals.into_iter());

        let t1 = line_eq.time_at(y0) as f64;
        let t3 = line_eq.time_at(yf) as f64;

        let theta = t1;
        let tau = t3 - t1;
        let k = yf - y0;

        Some(FirstOrderModel { k, tau, theta })
    }
}
