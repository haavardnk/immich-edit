use chrono::{DateTime, Utc};

use super::{JobRecord, JobStatus};

impl JobRecord {
    pub(super) fn with_eta(mut self, now: DateTime<Utc>) -> Self {
        self.eta_secs = eta_secs(&self, now);
        self.item_ms = item_ms(&self);
        self
    }
}

fn parse(raw: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .ok()
        .map(|at| at.with_timezone(&Utc))
}

struct Progress {
    progressed: DateTime<Utc>,
    elapsed: i64,
    settled: i64,
}

fn progress(job: &JobRecord) -> Option<Progress> {
    if job.status != JobStatus::Running {
        return None;
    }
    let started = parse(job.started_at.as_deref()?)?;
    let progressed = parse(&job.updated_at)?;
    let settled = job.completed + job.failed - job.started_done;
    let elapsed = (progressed - started).num_milliseconds();
    (settled > 0 && elapsed > 0).then_some(Progress {
        progressed,
        elapsed,
        settled,
    })
}

fn item_ms(job: &JobRecord) -> Option<u64> {
    let Progress {
        elapsed, settled, ..
    } = progress(job)?;
    u64::try_from(elapsed / settled).ok()
}

fn eta_secs(job: &JobRecord, now: DateTime<Utc>) -> Option<u64> {
    let Progress {
        progressed,
        elapsed,
        settled,
    } = progress(job)?;
    let remaining = job.total - job.completed - job.failed;
    if remaining <= 0 {
        return None;
    }
    let projected = elapsed.checked_mul(remaining)? / settled;
    let waited = (now - progressed).num_milliseconds().max(0);
    let left = u64::try_from(projected - waited).unwrap_or(0);
    Some(left.div_ceil(1000))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::job_store::tests::{create, store};
    use chrono::TimeDelta;
    use serde_json::json;
    use uuid::Uuid;

    fn at(raw: &str) -> DateTime<Utc> {
        parse(raw).unwrap()
    }

    fn running(
        total: i64,
        done: i64,
        started_done: i64,
        started: &str,
        progressed: &str,
    ) -> JobRecord {
        JobRecord {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            server_epoch: 1,
            kind: "test".into(),
            status: JobStatus::Running,
            target: json!(null),
            params: json!(null),
            total,
            completed: done,
            failed: 0,
            cancelled_at: None,
            created_at: started.into(),
            updated_at: progressed.into(),
            started_at: Some(started.into()),
            started_done,
            eta_secs: None,
            item_ms: None,
        }
    }

    #[test]
    fn the_rate_since_the_start_projects_the_remaining_items() {
        let job = running(10, 4, 0, "2026-01-01T00:00:00Z", "2026-01-01T00:00:08Z");
        assert_eq!(eta_secs(&job, at("2026-01-01T00:00:08Z")), Some(12));
        assert_eq!(eta_secs(&job, at("2026-01-01T00:00:11.5Z")), Some(9));
        assert_eq!(eta_secs(&job, at("2026-01-01T00:01:00Z")), Some(0));
    }

    #[test]
    fn items_settled_before_the_start_do_not_count_towards_the_rate() {
        let job = running(10, 6, 4, "2026-01-01T00:00:00Z", "2026-01-01T00:00:10Z");
        assert_eq!(eta_secs(&job, at("2026-01-01T00:00:10Z")), Some(20));
        assert_eq!(item_ms(&job), Some(5000));
    }

    #[test]
    fn the_item_rate_is_the_elapsed_time_per_settled_item() {
        let job = running(10, 4, 0, "2026-01-01T00:00:00Z", "2026-01-01T00:00:08Z");
        assert_eq!(item_ms(&job), Some(2000));
        let done = running(4, 4, 0, "2026-01-01T00:00:00Z", "2026-01-01T00:00:08Z");
        assert_eq!(
            (item_ms(&done), eta_secs(&done, at("2026-01-01T00:00:08Z"))),
            (Some(2000), None)
        );
        let mut queued = job;
        queued.status = JobStatus::Pending;
        assert_eq!(item_ms(&queued), None);
    }

    #[test]
    fn there_is_no_eta_without_progress_or_while_not_running() {
        let fresh = running(10, 2, 2, "2026-01-01T00:00:00Z", "2026-01-01T00:00:00Z");
        assert_eq!(eta_secs(&fresh, at("2026-01-01T00:00:05Z")), None);
        assert_eq!(item_ms(&fresh), None);
        let mut queued = running(10, 4, 0, "2026-01-01T00:00:00Z", "2026-01-01T00:00:08Z");
        queued.status = JobStatus::Pending;
        assert_eq!(eta_secs(&queued, at("2026-01-01T00:00:08Z")), None);
        queued.status = JobStatus::Running;
        queued.started_at = None;
        assert_eq!(eta_secs(&queued, at("2026-01-01T00:00:08Z")), None);
    }

    #[tokio::test]
    async fn a_running_job_reports_an_eta_once_an_item_settles() {
        let store = store().await;
        let job = create(&store, &["a", "b", "c"]).await;
        assert_eq!(job.eta_secs, None);
        let first = store.claim_next_item().await.unwrap().unwrap();
        let claimed = store.get_job(job.id).await.unwrap().unwrap();
        assert_eq!(
            (claimed.started_done, claimed.eta_secs, claimed.item_ms),
            (0, None, None)
        );
        let started = at(claimed.started_at.as_deref().unwrap());
        let backdated = (started - TimeDelta::seconds(30)).to_rfc3339();
        sqlx::query("UPDATE jobs SET started_at = ? WHERE id = ?")
            .bind(&backdated)
            .bind(job.id.to_string())
            .execute(&store.pool)
            .await
            .unwrap();
        store.complete_item(first.id, &json!({})).await.unwrap();
        let progressed = store.get_job(job.id).await.unwrap().unwrap();
        let eta = progressed.eta_secs.unwrap();
        assert!((59..=61).contains(&eta), "{eta}");
        let payload = serde_json::to_value(&progressed).unwrap();
        assert_eq!(payload["eta_secs"], json!(eta));
        let item_ms = progressed.item_ms.unwrap();
        assert!((29_000..=31_000).contains(&item_ms), "{item_ms}");
        assert_eq!(payload["item_ms"], json!(item_ms));
        assert!(payload.get("started_at").is_none());
        let listed = store.list_jobs(Uuid::nil(), 10).await.unwrap();
        assert_eq!(
            listed[0].eta_secs.map(|secs| secs.abs_diff(eta) <= 1),
            Some(true)
        );
    }
}
