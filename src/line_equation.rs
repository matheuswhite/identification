use aule::prelude::Signal;

pub struct LineEquation {
    x0: f32,
    y0: f64,
    m: f64,
}

impl LineEquation {
    pub fn new(x0: f32, y0: f64, m: f64) -> Self {
        Self { x0, y0, m }
    }

    #[allow(unused)]
    pub fn value_at(&self, x: f32) -> f64 {
        self.m * (x - self.x0) as f64 + self.y0
    }

    pub fn time_at(&self, y: f64) -> f32 {
        ((y - self.y0) / self.m) as f32 + self.x0
    }

    pub fn from_signals_with_maximum_slope(signals: impl Iterator<Item = Signal<f64>>) -> Self {
        let signals = signals.collect::<Vec<_>>();
        let mut max_slope = 0.0f64;
        let mut idx = 0;

        for i in 1..(signals.len() - 1) {
            let left = signals[i - 1];
            let center = signals[i];
            let right = signals[i + 1];

            let h1 = center.delta.sim_time().as_secs_f64() - left.delta.sim_time().as_secs_f64();
            let h2 = right.delta.sim_time().as_secs_f64() - center.delta.sim_time().as_secs_f64();

            let slope = (h2 / (h1 + h2)) * ((center.value - left.value) / h1)
                + (h1 / (h1 + h2)) * ((right.value - center.value) / h2);

            if slope.abs() > max_slope.abs() {
                max_slope = slope;
                idx = i;
            }
        }

        Self::new(
            signals[idx].delta.sim_time().as_secs_f32(),
            signals[idx].value,
            max_slope,
        )
    }
}
