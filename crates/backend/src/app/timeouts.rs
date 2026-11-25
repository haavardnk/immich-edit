use std::time::Duration;

use crate::config::Config;

pub fn export_request_timeout(config: &Config) -> Duration {
    phase_sum(config.original_timeout_secs, config.export_timeout_secs)
}

fn phase_sum(original_secs: u64, export_secs: u64) -> Duration {
    let bake_wait = export_secs;
    let render_and_upload = export_secs;
    Duration::from_secs(
        original_secs
            .saturating_add(bake_wait)
            .saturating_add(render_and_upload),
    )
}

#[cfg(test)]
mod tests {
    use super::phase_sum;
    use std::time::Duration;

    #[test]
    fn an_export_has_room_for_a_full_bake_wait_and_a_full_render() {
        assert_eq!(phase_sum(120, 300), Duration::from_secs(720));
        assert_eq!(phase_sum(u64::MAX, 1), Duration::from_secs(u64::MAX));
    }
}
