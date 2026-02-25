use anyhow::{Context, Result};

use crate::observability::SessionEvent;

use super::super::summary::SessionSummarySegment;
use super::RedisSessionBackend;

impl RedisSessionBackend {
    pub(crate) async fn append_summary_segment(
        &self,
        session_id: &str,
        max_segments: usize,
        segment: &SessionSummarySegment,
    ) -> Result<()> {
        let key = self.summary_key(session_id);
        let encoded =
            serde_json::to_string(segment).context("failed to encode summary segment for redis")?;
        let max_segments_i64 = super::backend::usize_to_i64_saturating(max_segments.max(1));
        let ttl_secs = self.ttl_secs;

        self.run_pipeline::<(), _>("append_summary_segment", || {
            let mut pipe = redis::pipe();
            pipe.atomic();
            pipe.cmd("RPUSH").arg(&key).arg(&encoded).ignore();
            pipe.cmd("LTRIM")
                .arg(&key)
                .arg(-max_segments_i64)
                .arg(-1)
                .ignore();
            if let Some(ttl) = ttl_secs {
                pipe.cmd("EXPIRE").arg(&key).arg(ttl).ignore();
            }
            pipe
        })
        .await?;
        tracing::debug!(
            event = SessionEvent::SessionSummarySegmentAppended.as_str(),
            session_id,
            max_segments,
            ttl_secs = ?ttl_secs,
            "valkey session summary segment appended"
        );
        Ok(())
    }

    pub(crate) async fn get_recent_summary_segments(
        &self,
        session_id: &str,
        limit: usize,
    ) -> Result<Vec<SessionSummarySegment>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let key = self.summary_key(session_id);
        let limit_i64 = super::backend::usize_to_i64_saturating(limit);
        let payloads = self
            .run_command::<Vec<String>, _>("get_recent_summary_segments", || {
                let mut cmd = redis::cmd("LRANGE");
                cmd.arg(&key).arg(-limit_i64).arg(-1);
                cmd
            })
            .await?;
        let mut out = Vec::with_capacity(payloads.len());
        let mut invalid_payloads = 0usize;
        for payload in payloads {
            match serde_json::from_str::<SessionSummarySegment>(&payload) {
                Ok(segment) => out.push(segment),
                Err(error) => {
                    invalid_payloads += 1;
                    tracing::warn!(
                        event = SessionEvent::SessionSummarySegmentsLoaded.as_str(),
                        session_id,
                        error = %error,
                        "invalid session summary payload in redis"
                    );
                }
            }
        }
        tracing::debug!(
            event = SessionEvent::SessionSummarySegmentsLoaded.as_str(),
            session_id,
            requested_limit = limit,
            loaded_segments = out.len(),
            invalid_payloads,
            "valkey session summary segments loaded"
        );
        Ok(out)
    }

    pub(crate) async fn get_summary_len(&self, session_id: &str) -> Result<usize> {
        let key = self.summary_key(session_id);
        let segment_count = self
            .run_command::<usize, _>("get_summary_len", || {
                let mut cmd = redis::cmd("LLEN");
                cmd.arg(&key);
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionSummarySegmentsLoaded.as_str(),
            session_id,
            loaded_segments = segment_count,
            count_only = true,
            "valkey session summary segment count loaded"
        );
        Ok(segment_count)
    }

    pub(crate) async fn clear_summary(&self, session_id: &str) -> Result<()> {
        let key = self.summary_key(session_id);
        let _ = self
            .run_command::<i64, _>("clear_summary", || {
                let mut cmd = redis::cmd("DEL");
                cmd.arg(&key);
                cmd
            })
            .await?;
        tracing::debug!(
            event = SessionEvent::SessionSummaryCleared.as_str(),
            session_id,
            "valkey session summary cleared"
        );
        Ok(())
    }
}
