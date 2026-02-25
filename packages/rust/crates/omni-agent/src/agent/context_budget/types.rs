use crate::config::ContextBudgetStrategy;
use crate::session::ChatMessage;

pub(crate) const SESSION_SUMMARY_MESSAGE_NAME: &str = "session.summary.segment";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MessageClass {
    Non,
    Regular,
    Summary,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ContextBudgetClassStats {
    pub input_messages: usize,
    pub kept_messages: usize,
    pub truncated_messages: usize,
    pub input_tokens: usize,
    pub kept_tokens: usize,
    pub truncated_tokens: usize,
}

impl ContextBudgetClassStats {
    pub(super) fn record_input(&mut self, tokens: usize) {
        self.input_messages = self.input_messages.saturating_add(1);
        self.input_tokens = self.input_tokens.saturating_add(tokens);
    }

    pub(super) fn record_kept(&mut self, original_tokens: usize, kept_tokens: usize) {
        self.kept_messages = self.kept_messages.saturating_add(1);
        self.kept_tokens = self.kept_tokens.saturating_add(kept_tokens);
        if kept_tokens < original_tokens {
            self.truncated_messages = self.truncated_messages.saturating_add(1);
            self.truncated_tokens = self
                .truncated_tokens
                .saturating_add(original_tokens.saturating_sub(kept_tokens));
        }
    }

    pub fn dropped_messages(&self) -> usize {
        self.input_messages.saturating_sub(self.kept_messages)
    }

    pub fn dropped_tokens(&self) -> usize {
        self.input_tokens.saturating_sub(self.kept_tokens)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ContextBudgetReport {
    pub strategy: ContextBudgetStrategy,
    pub budget_tokens: usize,
    pub reserve_tokens: usize,
    pub effective_budget_tokens: usize,
    pub pre_messages: usize,
    pub post_messages: usize,
    pub pre_tokens: usize,
    pub post_tokens: usize,
    pub non_system: ContextBudgetClassStats,
    pub regular_system: ContextBudgetClassStats,
    pub summary_system: ContextBudgetClassStats,
}

impl ContextBudgetReport {
    pub(super) fn new(
        strategy: ContextBudgetStrategy,
        budget_tokens: usize,
        reserve_tokens: usize,
        effective_budget_tokens: usize,
    ) -> Self {
        Self {
            strategy,
            budget_tokens,
            reserve_tokens,
            effective_budget_tokens,
            pre_messages: 0,
            post_messages: 0,
            pre_tokens: 0,
            post_tokens: 0,
            non_system: ContextBudgetClassStats::default(),
            regular_system: ContextBudgetClassStats::default(),
            summary_system: ContextBudgetClassStats::default(),
        }
    }

    pub(super) fn class_mut(&mut self, class: MessageClass) -> &mut ContextBudgetClassStats {
        match class {
            MessageClass::Non => &mut self.non_system,
            MessageClass::Regular => &mut self.regular_system,
            MessageClass::Summary => &mut self.summary_system,
        }
    }
}

#[derive(Clone)]
pub(super) struct IndexedMessage {
    pub index: usize,
    pub class: MessageClass,
    pub original_tokens: usize,
    pub message: ChatMessage,
}

#[derive(Clone)]
pub(super) struct SelectedMessage {
    pub index: usize,
    pub class: MessageClass,
    pub original_tokens: usize,
    pub kept_tokens: usize,
    pub message: ChatMessage,
}

pub(super) struct ClassifiedMessages {
    pub regular: Vec<IndexedMessage>,
    pub summary: Vec<IndexedMessage>,
    pub non: Vec<IndexedMessage>,
}

pub(crate) struct ContextBudgetPruneResult {
    pub messages: Vec<ChatMessage>,
    pub report: ContextBudgetReport,
}
