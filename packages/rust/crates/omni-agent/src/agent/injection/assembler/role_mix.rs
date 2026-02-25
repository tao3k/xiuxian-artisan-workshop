use std::collections::HashSet;

use xiuxian_qianhuan::{
    InjectionMode, InjectionPolicy, PromptContextBlock, PromptContextCategory, RoleMixProfile,
    RoleMixRole,
};

pub(super) fn select_role_mix(
    policy: &InjectionPolicy,
    blocks: &[PromptContextBlock],
) -> RoleMixProfile {
    let mut roles = collect_role_candidates(blocks);
    if roles.is_empty() {
        roles.push(default_role_mix_role());
    }
    build_role_mix_profile(policy.mode, roles)
}

fn collect_role_candidates(blocks: &[PromptContextBlock]) -> Vec<RoleMixRole> {
    let mut roles = Vec::new();
    let mut seen = HashSet::new();

    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[PromptContextCategory::Safety, PromptContextCategory::Policy],
        "governance_guardian",
        0.36,
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[
            PromptContextCategory::MemoryRecall,
            PromptContextCategory::WindowSummary,
        ],
        "memory_strategist",
        0.31,
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[PromptContextCategory::SessionXml],
        "session_context_curator",
        0.27,
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[PromptContextCategory::Knowledge],
        "knowledge_synthesizer",
        0.33,
    );
    maybe_push_role_for_categories(
        blocks,
        &mut roles,
        &mut seen,
        &[
            PromptContextCategory::Reflection,
            PromptContextCategory::RuntimeHint,
        ],
        "reflection_optimizer",
        0.29,
    );

    roles
}

fn build_role_mix_profile(mode: InjectionMode, roles: Vec<RoleMixRole>) -> RoleMixProfile {
    match mode {
        InjectionMode::Single => {
            let primary = roles.first().cloned().unwrap_or_else(default_role_mix_role);
            RoleMixProfile {
                profile_id: "role_mix.single.v1".to_string(),
                roles: vec![primary.clone()],
                rationale: format!(
                    "policy.mode=single selected deterministic primary role `{}`",
                    primary.role
                ),
            }
        }
        InjectionMode::Classified => RoleMixProfile {
            profile_id: "role_mix.classified.v1".to_string(),
            rationale: format!(
                "policy.mode=classified selected {} role domains from retained blocks",
                roles.len()
            ),
            roles,
        },
        InjectionMode::Hybrid => RoleMixProfile {
            profile_id: "role_mix.hybrid.v1".to_string(),
            rationale: format!(
                "policy.mode=hybrid selected {} role domains for mixed-context synthesis",
                roles.len()
            ),
            roles,
        },
    }
}

fn maybe_push_role_for_categories(
    blocks: &[PromptContextBlock],
    roles: &mut Vec<RoleMixRole>,
    seen: &mut HashSet<&'static str>,
    categories: &[PromptContextCategory],
    role: &'static str,
    weight: f32,
) {
    if has_any_category(blocks, categories) {
        push_role(roles, seen, role, weight);
    }
}

fn has_any_category(blocks: &[PromptContextBlock], categories: &[PromptContextCategory]) -> bool {
    blocks
        .iter()
        .any(|block| categories.contains(&block.category))
}

fn default_role_mix_role() -> RoleMixRole {
    RoleMixRole {
        role: "session_context_curator".to_string(),
        weight: 1.0,
    }
}

fn push_role(
    roles: &mut Vec<RoleMixRole>,
    seen: &mut HashSet<&'static str>,
    role: &'static str,
    weight: f32,
) {
    if seen.insert(role) {
        roles.push(RoleMixRole {
            role: role.to_string(),
            weight,
        });
    }
}
