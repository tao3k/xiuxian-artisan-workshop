---
type: knowledge
metadata:
  title: Scheduling Rules
  routing_keywords:
    - "carryover"
    - "cognitive load"
    - "task prioritization"
  intents:
    - "Apply anti-overload constraints"
    - "Prioritize stale commitments"
---

# Scheduling Rules

## Carryover Definition

Carryover means a task was scheduled but not completed in the intended window.
Repeated carryover is a hard signal of planning quality failure.

## Anti-Overload Rule

Do not schedule many high-cognitive tasks in one block.
If historical carryover is high, reduce scope before adding new commitments.

## Priority Discipline

Prefer this order:

1. Clear stale commitments.
2. Schedule one meaningful progression task.
3. Add optional low-risk tasks only if capacity remains.

## Reminder and Time Integrity

Use explicit local time in user-facing output.
Avoid leaking internal metadata or raw system serialization formats in user replies.
