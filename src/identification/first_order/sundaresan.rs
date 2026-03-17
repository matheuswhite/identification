use crate::identification::first_order::{
    FirstOrderIdentification, FirstOrderModel, find_time_at_value,
};
use aule::prelude::Signal;

pub struct Sundaresan;

impl FirstOrderIdentification for Sundaresan {
    fn from_step_response(&self, signals: Vec<Signal<f64>>) -> Option<FirstOrderModel> {
        if signals.len() < 2 {
            return None;
        }

        let y0 = signals.first().unwrap().value;
        let yf = signals.last().unwrap().value;
        let y353 = yf * 0.353 + y0;
        let y853 = yf * 0.853 + y0;

        let t1 = find_time_at_value(signals.iter(), y353)
            .delta
            .sim_time()
            .as_secs_f64();
        let t2 = find_time_at_value(signals.iter(), y853)
            .delta
            .sim_time()
            .as_secs_f64();
        let tau = 0.67 * (t2 - t1);
        let theta = 1.3 * t1 - 0.29 * t2;
        let k = yf - y0;

        Some(FirstOrderModel { k, tau, theta })
    }
}
