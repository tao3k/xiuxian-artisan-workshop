use std::sync::Arc;

use tokio::sync::mpsc;

use crate::agent::Agent;
use crate::channels::traits::Channel;
use crate::channels::traits::ChannelMessage;

pub(super) async fn forward(
    msg: ChannelMessage,
    channel: &Arc<dyn Channel>,
    agent: &Arc<Agent>,
    foreground_tx: &mpsc::Sender<ChannelMessage>,
) -> bool {
    let session_key = msg.session_key.clone();
    let channel_name = msg.channel.clone();
    let recipient = msg.recipient.clone();
    let admission = agent.evaluate_downstream_admission();
    if !admission.admitted {
        if let Some(reason) = admission.reason {
            tracing::warn!(
                event = "telegram.foreground.admission_reject",
                reason = reason.as_str(),
                session_key = %session_key,
                channel = %channel_name,
                recipient = %recipient,
                llm_saturation_pct = ?admission.snapshot.llm.map(|state| state.saturation_pct),
                embedding_saturation_pct = ?admission.snapshot.embedding.map(|state| state.saturation_pct),
                llm_reject_threshold_pct = admission.llm_reject_threshold_pct,
                embedding_reject_threshold_pct = admission.embedding_reject_threshold_pct,
                "telegram foreground turn rejected by downstream admission control"
            );
            if let Err(error) = channel.send(reason.user_message(), &recipient).await {
                tracing::warn!(
                    error = %error,
                    session_key = %session_key,
                    recipient = %recipient,
                    "failed to send telegram admission rejection notice"
                );
            }
        }
        return true;
    }

    if foreground_tx.send(msg).await.is_err() {
        tracing::error!("Foreground dispatcher is unavailable");
        return false;
    }
    true
}
