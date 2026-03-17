use crate::identification::first_order::{
    FirstOrderIdentification, FirstOrderModel, find_time_at_value,
};
use aule::prelude::Signal;

pub struct Smith;

impl FirstOrderIdentification for Smith {
    fn from_step_response(&self, signals: Vec<Signal<f64>>) -> Option<FirstOrderModel> {
        if signals.len() < 2 {
            return None;
        }

        let y0 = signals.first().unwrap().value;
        let yf = signals.last().unwrap().value;
        let y283 = yf * 0.283 + y0;
        let y632 = yf * 0.632 + y0;

        let t1 = find_time_at_value(signals.iter(), y283)
            .delta
            .sim_time()
            .as_secs_f64();
        let t2 = find_time_at_value(signals.iter(), y632)
            .delta
            .sim_time()
            .as_secs_f64();
        let tau = 1.5 * (t2 - t1);
        let theta = t2 - tau;
        let k = yf - y0;

        Some(FirstOrderModel { k, tau, theta })
    }
}
