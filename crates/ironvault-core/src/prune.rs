//! Pruning functionality for IronVault

use crate::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashSet;
use tracing::{info, span, Level};

/// Pruning policy
#[derive(Debug, Clone)]
pub struct PrunePolicy {
    pub keep_hourly: usize,
    pub keep_daily: usize,
    pub keep_weekly: usize,
    pub keep_monthly: usize,
    pub keep_yearly: usize,
}

impl PrunePolicy {
    pub fn new(
        keep_hourly: usize,
        keep_daily: usize,
        keep_weekly: usize,
        keep_monthly: usize,
        keep_yearly: usize,
    ) -> Self {
        Self {
            keep_hourly,
            keep_daily,
            keep_weekly,
            keep_monthly,
            keep_yearly,
        }
    }

    /// Calculate which snapshots to keep
    pub fn calculate_keep_list(&self, snapshots: &[DateTime<Utc>]) -> HashSet<DateTime<Utc>> {
        let mut keep = HashSet::new();
        let mut sorted = snapshots.to_vec();
        sorted.sort_by(|a, b| b.cmp(a)); // Newest first

        // Keep hourly
        let hourly_interval = Duration::hours(1);
        Self::keep_by_interval(&mut keep, &sorted, hourly_interval, self.keep_hourly);

        // Keep daily
        let daily_interval = Duration::days(1);
        Self::keep_by_interval(&mut keep, &sorted, daily_interval, self.keep_daily);

        // Keep weekly
        let weekly_interval = Duration::weeks(1);
        Self::keep_by_interval(&mut keep, &sorted, weekly_interval, self.keep_weekly);

        // Keep monthly
        let monthly_interval = Duration::days(30);
        Self::keep_by_interval(&mut keep, &sorted, monthly_interval, self.keep_monthly);

        // Keep yearly
        let yearly_interval = Duration::days(365);
        Self::keep_by_interval(&mut keep, &sorted, yearly_interval, self.keep_yearly);

        keep
    }

    fn keep_by_interval(
        keep: &mut HashSet<DateTime<Utc>>,
        snapshots: &[DateTime<Utc>],
        interval: Duration,
        count: usize,
    ) {
        let mut kept = 0;
        let mut last_kept: Option<DateTime<Utc>> = None;

        for snapshot in snapshots {
            if keep.contains(snapshot) {
                continue;
            }

            match last_kept {
                None => {
                    keep.insert(*snapshot);
                    last_kept = Some(*snapshot);
                    kept += 1;
                }
                Some(last) => {
                    if *snapshot < last - interval {
                        keep.insert(*snapshot);
                        last_kept = Some(*snapshot);
                        kept += 1;
                    }
                }
            }

            if kept >= count {
                break;
            }
        }
    }
}

/// Prune result
#[derive(Debug, Clone)]
pub struct PruneResult {
    pub deleted_count: usize,
    pub kept_count: usize,
    pub freed_space: u64,
}

/// Execute pruning
pub fn prune_snapshots(
    snapshots: Vec<(String, DateTime<Utc>)>,
    policy: &PrunePolicy,
) -> Result<PruneResult> {
    let _span = span!(Level::INFO, "prune_snapshots");

    let dates: Vec<_> = snapshots.iter().map(|(_, d)| *d).collect();
    let to_keep = policy.calculate_keep_list(&dates);

    let mut result = PruneResult {
        deleted_count: 0,
        kept_count: 0,
        freed_space: 0,
    };

    for (name, date) in snapshots {
        if to_keep.contains(&date) {
            result.kept_count += 1;
            info!(snapshot = name, "Keeping snapshot");
        } else {
            result.deleted_count += 1;
            info!(snapshot = name, "Deleting snapshot");
            // In real implementation, would delete the snapshot file
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prune_policy() {
        let policy = PrunePolicy::new(24, 7, 4, 6, 1);

        let now = Utc::now();
        let snapshots = vec![
            (now - Duration::hours(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            (now - Duration::hours(2))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
            (now - Duration::days(1))
                .date_naive()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc(),
        ];

        let keep = policy.calculate_keep_list(&snapshots);
        assert!(keep.len() > 0);
    }
}
