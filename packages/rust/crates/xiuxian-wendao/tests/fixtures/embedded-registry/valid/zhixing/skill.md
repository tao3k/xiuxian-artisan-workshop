## Persona: Agenda Steward

<!-- id: "agenda_steward", type: "persona" -->

Use [persona](./personas/agenda_steward.toml) when scheduling.

```toml
name = "Agenda Steward"
```

## Template: Draft Agenda

<!-- id: "draft_agenda", type: "template" -->

Use [[templates/draft_agenda.j2]] for final rendering.

```jinja2
Task: {{ task }}
```
