use aule::prelude::Signal;

use crate::identification::{
    first_order::find_time_at_value,
    second_order::{SecondOrderIdentification, SecondOrderModel},
};

pub struct Mollenkamp;

impl SecondOrderIdentification for Mollenkamp {
    fn from_step_response(&self, signals: Vec<Signal<f64>>) -> Option<SecondOrderModel> {
        let y0 = signals.first().unwrap().value;
        let yf = signals.last().unwrap().value;
        let y15 = yf * 0.15 + y0;
        let y45 = yf * 0.45 + y0;
        let y75 = yf * 0.75 + y0;

        let t1 = find_time_at_value(signals.iter(), y15)
            .delta
            .sim_time()
            .as_secs_f64();
        let t2 = find_time_at_value(signals.iter(), y45)
            .delta
            .sim_time()
            .as_secs_f64();
        let t3 = find_time_at_value(signals.iter(), y75)
            .delta
            .sim_time()
            .as_secs_f64();

        let x = (t2 - t1) / (t3 - t1);

        let zeta = (0.0805 - 5.547 * (0.475 - x).powi(2)) / (x - 0.356);

        let f2 = if zeta < 1.0 {
            0.708 * 2.811_f64.powf(zeta)
        } else {
            2.6 * zeta - 0.60
        };

        let omega_n = f2 / (t3 - t1);

        let f3 = 0.922 * 1.66_f64.powf(zeta);

        let theta = t2 - (f3 / omega_n);

        Some(SecondOrderModel {
            k: yf,
            theta,
            zeta,
            omega_n,
        })
    }
}
