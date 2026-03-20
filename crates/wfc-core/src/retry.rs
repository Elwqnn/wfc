/// Run `attempts` solvers in parallel with varied seeds, return first success.
#[cfg(feature = "parallel")]
pub fn parallel_solve(
    sample: &crate::Sample,
    config: &crate::config::Config,
    attempts: usize,
) -> Option<Vec<crate::Color>> {
    use rayon::prelude::*;

    use crate::RunOutcome;
    use crate::solver::Wfc;

    let base_seed = config.seed.unwrap_or(0);

    (0..attempts).into_par_iter().find_map_any(|i| {
        let cfg = crate::config::Config {
            seed: Some(base_seed.wrapping_add(i as u64)),
            ..config.clone()
        };
        let mut wfc = Wfc::new(sample, cfg);
        match wfc.run() {
            RunOutcome::Complete => Some(wfc.render()),
            RunOutcome::Contradiction => None,
        }
    })
}
