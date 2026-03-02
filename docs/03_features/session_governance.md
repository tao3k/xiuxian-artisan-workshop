---
type: knowledge
title: "Feature: Session Governance & Human-Centric Interaction"
category: "features"
tags:
  - session
  - ux
  - privacy
  - interrupt
metadata:
  title: "Feature: Session Governance & Human-Centric Interaction"
---

# Feature: Session Governance & Human-Centric Interaction

This feature provides robust lifecycle management and interactive control for multi-user agent sessions (Telegram/Discord).

## 1. Session Isolation (Scoping)

The system supports multiple isolation modes to ensure privacy and collaborative flexibility:

- **`user` (Default)**: Private context per user across all chats.
- **`chat`**: Shared context for all users within a single group.
- **`chat_user`**: Isolate by both group and individual sender.

Users can toggle these modes via the `/session scope` (alias of `/session partition`) command.

## 2. Interactive Controls (QueueMode)

### 2.1 The Interrupt Mechanism

Implemented via the `InterruptController`, this allows users to immediately halt a slow or incorrect LLM reasoning process.

- **Trigger**: Any new message from the same user while a turn is in-flight.
- **Mechanism**: Sends a cancellation signal to the `tokio` task and aborts the upstream LLM request.

### 2.2 Human-in-the-Loop (HITL)

Restricted tools (e.g., system commands) require explicit approval via platform-native buttons before execution.

## 3. Lifecycle & Freshness

### 3.1 Idle Reset

Sessions are automatically archived and reset after a configurable period of inactivity (`reset_idle_timeout_mins`).

### 3.2 Metadata UX

- **Auto-Titles**: The first 60 characters of a session's first message are extracted as the display title.
- **Persistence**: Partition modes can be persisted to the user's `xiuxian.toml` via the `OMNI_AGENT_SESSION_PERSISTENCE` toggle.

## 4. Verification

- Verified via `cargo nextest` and live Telegram/Discord webhook probes.

## 5. Runtime Performance Notes

- Persona resolution now uses a read-through registry cache with Wendao-backed fallback on cache miss.
- Session turns avoid repeated persona file scans after first lookup because profiles are cached in-memory by ID.
