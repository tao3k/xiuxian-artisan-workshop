use num_traits::ToPrimitive;

use super::{MemoryRecallInput, MemoryRecallPlan};

/// Derive dynamic memory-recall parameters from current context pressure.
pub(crate) fn plan_memory_recall(input: MemoryRecallInput) -> MemoryRecallPlan {
    let mut k1 = input.base_k1.max(1);
    let mut k2 = input.base_k2.max(1).min(k1);
    let mut lambda = clamp_lambda(input.base_lambda);
    let mut min_score = 0.08_f32;
    let mut max_context_chars = (320 + k2.saturating_mul(220)).clamp(480, 1_800);

    let effective_budget_tokens = input.context_budget_tokens.map(|budget| {
        budget
            .saturating_sub(input.context_budget_reserve_tokens)
            .max(1)
    });
    let budget_pressure = effective_budget_tokens.map_or(0.0, |effective| {
        ratio_usize_as_f32(input.context_tokens_before_recall, effective)
    });
    let window_pressure = match input.window_max_turns {
        Some(max_turns) if max_turns > 0 => {
            ratio_usize_as_f32(input.active_turns_estimate, max_turns)
        }
        _ => 0.0,
    };

    if budget_pressure >= 1.0 {
        k2 = k2.clamp(1, 2);
        k1 = k1.min(8).max(k2);
        lambda = (lambda + 0.2).clamp(0.0, 0.95);
        min_score = 0.20;
        max_context_chars = (300 + k2.saturating_mul(160)).clamp(320, 700);
    } else if budget_pressure >= 0.8 {
        k2 = k2.clamp(1, 3);
        k1 = k1.min(12).max(k2);
        lambda = (lambda + 0.1).clamp(0.0, 0.90);
        min_score = 0.15;
        max_context_chars = (420 + k2.saturating_mul(180)).clamp(420, 1_000);
    } else if budget_pressure <= 0.45
        && (window_pressure >= 0.75 || input.summary_segment_count > 0)
    {
        let boosted_k2_cap = input.base_k2.saturating_add(2).max(2);
        let boosted_k1_cap = input.base_k1.saturating_add(8).max(4);
        k2 = k2.saturating_add(1).min(boosted_k2_cap).max(1);
        k1 = k1.saturating_add(4).min(boosted_k1_cap).max(k2);
        lambda = (lambda - 0.05).clamp(0.0, 0.90);
        min_score = 0.05;
        max_context_chars = (420 + k2.saturating_mul(240)).clamp(640, 2_200);
    }

    MemoryRecallPlan {
        k1,
        k2,
        lambda,
        min_score,
        max_context_chars,
        budget_pressure,
        window_pressure,
        effective_budget_tokens,
    }
}

fn clamp_lambda(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.3
    }
}

fn ratio_usize_as_f32(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        return 0.0;
    }
    let numerator = numerator.to_f32().unwrap_or(f32::MAX);
    let denominator = denominator.to_f32().unwrap_or(f32::MAX);
    numerator / denominator
}
