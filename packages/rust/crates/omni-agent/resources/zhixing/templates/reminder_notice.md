---
type: knowledge
metadata:
  title: "reminder_notice"
---

_⏰ Agenda 提醒_
我是你的日程安排管家 _{{ persona_name_mdv2 }}_。
_任务名称:_ {{ task_title_mdv2 }}
{% if task_brief_mdv2 %}_任务说明:_ {{ task_brief_mdv2 }}{% endif %}
{% if scheduled_local_mdv2 %}_提醒时间:_ {{ scheduled_local_mdv2 }}{% endif %}
_任务编号:_ `{{ task_id_mdv2 }}`
_建议操作:_ 现在开始处理该任务，完成后请更新你的 Agenda 状态。
