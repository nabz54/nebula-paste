use nebula_paste::model::{Clip, MAX_ITEMS};
use std::{hint::black_box, time::Instant};
fn main() {
    println!("version,count,iterations,total_ms,us_per_query");
    for count in [1, 100, MAX_ITEMS] {
        let clips: Vec<_> = (0..count)
            .map(|i| {
                Clip::new(
                    "text/plain".into(),
                    format!(
                        "Réponse numéro {i} : serveur de démonstration. {}",
                        "notes ".repeat(100)
                    )
                    .into_bytes(),
                    i as i64,
                )
                .unwrap()
            })
            .collect();
        let start = Instant::now();
        let iterations = 200;
        for _ in 0..iterations {
            let n = clips
                .iter()
                .filter(|c| c.matches(black_box("reponse serveur"), None, false, ""))
                .count();
            assert_eq!(n, count);
            black_box(n);
        }
        let d = start.elapsed();
        println!(
            "{},{count},{iterations},{:.3},{:.3}",
            env!("CARGO_PKG_VERSION"),
            d.as_secs_f64() * 1000.0,
            d.as_secs_f64() * 1e6 / f64::from(iterations)
        );
    }
}
