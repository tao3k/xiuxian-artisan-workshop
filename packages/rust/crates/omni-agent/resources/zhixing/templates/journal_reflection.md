---
type: knowledge
metadata:
  title: "Journal Captured"
---

# Journal Captured

{% if qianhuan.persona and qianhuan.persona.name %}

> Steward: **{{ qianhuan.persona.name }}**
> {% endif %}
> {{ qianhuan.injected_context }}

- Manifested Task: **{{ task_title }}**
- Task ID: `{{ task_id }}`
- Journal ID: `{{ journal_id }}`

Next step: run `agenda.view` to verify execution order and time slots.
