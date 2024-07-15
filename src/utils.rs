use std::{fs::OpenOptions, io::Write, time::Duration};
use rand::{rngs::ThreadRng, Rng};

pub fn weighted_random_selection(
    probability_distribution: &[u32],
    random: &mut ThreadRng,
) -> usize {
    if probability_distribution.len() == 1 {
        return 0;
    }
    assert!(!probability_distribution.is_empty(), "there are no songs in the playlist");
    let mut sum = probability_distribution.iter().sum::<u32>();
    assert_ne!(
        sum,
        0,
        "exclude lyrics mode is enabled but all songs in the playlist are set to have lyrics"
    );
    for (i, p) in probability_distribution.iter().enumerate() {
        if random.gen_bool(*p as f64 / sum as f64) {
            return i;
        }
        sum -= p;
    }
    unreachable!();
}

pub fn format_duration(duration: &Duration) -> String {
    if duration.as_secs() % 60 < 10 {
        format!("{}:0{}", duration.as_secs() / 60, duration.as_secs() % 60)
    } else {
        format!("{}:{}", duration.as_secs() / 60, duration.as_secs() % 60)
    }
}

pub fn write_to_file(file: &str, contents: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file)
        .expect("Failed to open file");
    file.write_all(contents.as_bytes())
        .expect("Failed to write to file");
}
