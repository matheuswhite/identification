use aule::prelude::*;
use clap::{Parser, Subcommand};
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Systems,
    Identification,
    GenerateSamples,
}

fn main() {
    let args = Args::parse();

    std::fs::create_dir_all("output").expect("Failed to create output directory");

    match args.commands {
        Commands::Systems => systems(),
        Commands::Identification => identification(),
        Commands::GenerateSamples => generate_samples(),
    }
}

fn systems() {
    use aule::s;

    let time = Time::new(0.001, 6.0);

    let mut plants = [
        (20.0f64 / ((s + 2.0) * (s + 10.0))).to_ss_controllable(RK4),
        (2.0f64 / (s + 2.0)).to_ss_controllable(RK4),
        ((2.0f64 * 15.0 * 200.0) / ((s + 2.0) * (s + 15.0) * (s + 200.0))).to_ss_controllable(RK4),
    ];
    let mut reference = Sinusoid::new(1.0, Duration::from_secs(1), 0.0);
    let mut plotter = Plotter::<4, f64>::new("Ident".to_string());

    for dt in time {
        let input = dt * reference.as_block();
        let mut output = [input; 4];
        for (i, plant) in plants.iter_mut().enumerate() {
            output[i + 1] = input * plant.as_block();
        }
        let _ = output.pack() * plotter.as_block();
    }

    plotter.display();
    plotter.join();
}

fn identification() {
    let files = [
        ("conjunto1", 10.0),
        ("conjunto2", 2.5),
        ("conjunto3", 2.5),
        ("conjunto4", 0.5),
        ("conjunto5", 1.5),
        ("conjunto6", 0.5),
        ("system3", 100.0),
        ("system6", 100.0),
        ("system9", 100.0),
        ("system12", 100.0),
        ("system15", 100.0),
    ];
    let first_order_methods: &[(&str, &dyn FirstOrderIdentification)] = &[
        ("Ziegler-Nichols", &ZieglerNichols),
        ("Hagglund", &Hagglund),
        ("Smith1", &Smith1),
        ("Sundaresan", &SundaresanKrishnaswamy),
    ];
    let second_order_methods: &[(&str, &dyn SecondOrderIdentification)] =
        &[("Mollenkamp", &Mollenkamp), ("Smith2", &Smith2)];

    for (filename, cutoff_freq) in files {
        println!("Processing file: {}.csv (f = {} Hz)", filename, cutoff_freq);

        for (method_name, method) in first_order_methods.iter() {
            std::fs::create_dir_all(format!("output/{}/", method_name))
                .expect("Failed to create output directory");

            if let Err(err) =
                identification_first_order(filename, cutoff_freq, *method, method_name)
            {
                eprintln!(
                    "Error processing {} with method {}: {}",
                    filename, method_name, err
                );
            }
        }

        for (method_name, method) in second_order_methods.iter() {
            std::fs::create_dir_all(format!("output/{}/", method_name))
                .expect("Failed to create output directory");

            if let Err(err) =
                identification_second_order(filename, cutoff_freq, *method, method_name)
            {
                eprintln!(
                    "Error processing {} with method {}: {}",
                    filename, method_name, err
                );
            }
        }
    }
}

fn identification_first_order(
    filename: &str,
    cutoff_freq: f64,
    method: &dyn FirstOrderIdentification,
    method_name: &str,
) -> Result<(), String> {
    let dt = 1e-3;

    let mut samples = FileSamples::from_csv(&format!("samples/{}.csv", filename), 0, 1)
        .expect("Failed to read CSV signal");

    let mut filter = LowPass::<f64>::new(cutoff_freq, Duration::from_secs_f64(dt as f64));
    let mut sample_clone = samples.clone();
    let filtered_samples = filter
        .filter(move |dt| (dt * sample_clone.as_block()).unpack())
        .collect::<Vec<_>>();

    let identification = method
        .from_step_response(filtered_samples.clone())
        .map_err(|err| format!("Failed to identify system: {:?}", err))?;
    println!("\t{} parameters: {}", method_name, identification);

    let time = EndlessTime::new(dt);
    let (detected_system, mut delay) = identification
        .try_into()
        .map_err(|err| format!("Fail to identify system: {:?}", err))?;
    let mut detected_system = detected_system.to_ss_controllable(RK4);
    let mut plotter = Plotter::<3, f64>::new(format!("{} - {}", filename, method_name));
    let mut step = Step::new(1.0);
    let mut iae = IAE::<f64>::default();
    let mut ise = ISE::<f64>::default();
    let mut itae = ITAE::<f64>::default();

    for (i, dt) in time.enumerate() {
        let sample_signal = dt * samples.as_block();
        if sample_signal.value.is_none() {
            break;
        };
        let sample_signal = sample_signal.map(|v| v.unwrap());
        let filtered_signal = filtered_samples[i];

        let input = dt * step.as_block();
        let output = input * detected_system.as_block();
        let output = output * delay.as_block();

        let error = output - sample_signal;
        let _ = error * iae.as_block() * ise.as_block() * itae.as_block();

        let _ = [sample_signal, filtered_signal, output].pack() * plotter.as_block();
    }

    println!("\t\tIAE: {}", iae.value());
    println!("\t\tISE: {}", ise.value());
    println!("\t\tITAE: {}", itae.value());
    plotter.display();

    plotter
        .save(format!("output/{}/{}.png", method_name, filename).as_str())
        .expect("Failed to save plot");

    Ok(())
}

fn identification_second_order(
    filename: &str,
    cutoff_freq: f64,
    method: &dyn SecondOrderIdentification,
    method_name: &str,
) -> Result<(), String> {
    let dt = 1e-3;

    let mut samples = FileSamples::from_csv(&format!("samples/{}.csv", filename), 0, 1)
        .expect("Failed to read CSV signal");
    let mut filter = LowPass::<f64>::new(cutoff_freq, Duration::from_secs_f64(dt as f64));
    let mut sample_clone = samples.clone();
    let filtered_samples = filter
        .filter(move |dt| (dt * sample_clone.as_block()).unpack())
        .collect::<Vec<_>>();

    let identification = method
        .from_step_response(filtered_samples.clone())
        .map_err(|err| format!("Failed to identify system: {:?}", err))?;
    println!("\t{} parameters: {}", method_name, identification);

    let time = EndlessTime::new(dt);
    let (detected_system, mut delay) = identification
        .try_into()
        .map_err(|err| format!("Failed to identify system: {:?}", err))?;
    let mut detected_system = detected_system.to_ss_controllable(RK4);
    let mut plotter = Plotter::<3, f64>::new(format!("{} - {}", filename, method_name));
    let mut step = Step::new(1.0);
    let mut iae = IAE::<f64>::default();
    let mut ise = ISE::<f64>::default();
    let mut itae = ITAE::<f64>::default();

    for (i, dt) in time.enumerate() {
        let sample_signal = dt * samples.as_block();
        if sample_signal.value.is_none() {
            break;
        }
        let sample_signal = sample_signal.map(|v| v.unwrap());
        let filtered_signal = filtered_samples[i];

        let input = dt * step.as_block();
        let output = input * detected_system.as_block();
        let output = output * delay.as_block();

        let error = output - sample_signal;
        let _ = error * iae.as_block() * ise.as_block() * itae.as_block();

        let _ = [sample_signal, filtered_signal, output].pack() * plotter.as_block();
    }

    println!("\t\tIAE: {}", iae.value());
    println!("\t\tISE: {}", ise.value());
    println!("\t\tITAE: {}", itae.value());
    plotter.display();

    plotter
        .save(format!("output/{}/{}.png", method_name, filename).as_str())
        .expect("Failed to save plot");

    Ok(())
}

fn generate_samples() {
    use aule::s;

    let system3: Tf<f64> =
        (2.0 * (15.0 * s + 1.0)) / ((20.0 * s + 1.0) * (s + 1.0) * (0.1f64 * s + 1.0).pow(2));
    let system6 = (0.17f64 * s + 1.0).pow(2) / (s * (s + 1.0).pow(2) * (0.028 * s + 1.0));
    let system9 = 1.0 / (s + 1.0).pow(2);
    let system12 =
        ((6.0 * s + 1.0) * (3.0 * s + 1.0)) / ((10.0 * s + 1.0) * (8.0 * s + 1.0) * (s + 1.0));
    let system15 = Tf::new(&[-1.0, 1.0], &[1.0, 1.0]);

    let mut systems: [(
        SS<RK4, f64>,
        Option<Delay<f64>>,
        &str,
        f64,
        Writter<1, f64>,
        Plotter<1, f64>,
    ); 5] = [
        (system3, None, "system3", 60.0),
        (system6, None, "system6", 10.0),
        (system9, Some(Duration::from_secs_f64(1.0)), "system9", 10.0),
        (
            system12,
            Some(Duration::from_secs_f64(0.3)),
            "system12",
            60.0,
        ),
        (system15, None, "system15", 10.0),
    ]
    .map(|(sys, d, name, max_time)| {
        (
            sys.to_ss_controllable(RK4),
            d.map(|d| Delay::new(d)),
            name,
            max_time,
            Writter::new(&format!("samples/{}.csv", name), ["signal"]),
            Plotter::new(name.to_owned()),
        )
    });

    let mut input = Step::new(1.0);

    let max_time = 60.0;
    for dt in Time::new(5e-3, max_time) {
        let input_signal = dt * input.as_block();

        for (sys, delay, _, max_time_sys, writter, plotter) in &mut systems {
            if dt.delta.sim_time().as_secs_f64() > *max_time_sys {
                continue;
            }

            let output = input_signal * sys.as_block();

            let output = match delay {
                Some(d) => output * d.as_block(),
                None => output,
            };

            let _ = output * writter.as_block();
            let _ = output * plotter.as_block();
        }

        println!(
            "{:.2}%...",
            dt.delta.sim_time().as_secs_f32() * 100.0 / max_time
        );
    }

    for (_, _, name, _, _, plotter) in &mut systems {
        plotter.display();
        std::fs::create_dir_all("output/samples/").expect("Failed to create output directory");
        let _ = plotter.save(&format!("output/samples/{}.png", name));
        plotter.join();
    }
}
