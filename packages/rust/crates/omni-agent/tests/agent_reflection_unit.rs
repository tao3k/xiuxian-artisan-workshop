//! Top-level integration harness for `agent::reflection`.

mod contracts {
    pub(crate) use omni_agent::{
        OmegaFallbackPolicy, OmegaRiskLevel, OmegaRoute, OmegaToolTrustClass,
    };
}

#[path = "../src/agent/reflection/mod.rs"]
mod reflection_impl;

mod agent {
    pub(crate) mod reflection {
        pub(crate) use crate::reflection_impl::{
            ReflectiveRuntime, ReflectiveRuntimeStage, build_turn_reflection, derive_policy_hint,
        };

        fn lint_symbol_probe() {
            let _ = std::mem::size_of::<crate::reflection_impl::ReflectiveRuntimeError>();
            let _ = std::mem::size_of::<crate::reflection_impl::PolicyHintDirective>();
            let _ = crate::reflection_impl::render_turn_reflection_block
                as fn(&crate::reflection_impl::TurnReflection) -> String;
            let _ = crate::reflection_impl::render_turn_reflection_for_memory
                as fn(&crate::reflection_impl::TurnReflection) -> String;
        }

        const _: fn() = lint_symbol_probe;

        mod tests {
            include!("agent/reflection.rs");
        }
    }
}
