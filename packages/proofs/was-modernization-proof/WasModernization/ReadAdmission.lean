import Std.Tactic

namespace WasModernization.ReadAdmission

structure Usage where
  commands : Nat
  items : Nat
  bytes : Nat
  deriving DecidableEq, Repr

def add (a b : Usage) : Usage :=
  ⟨a.commands + b.commands, a.items + b.items, a.bytes + b.bytes⟩

def bounded (u limits : Usage) : Prop :=
  u.commands ≤ limits.commands ∧ u.items ≤ limits.items ∧ u.bytes ≤ limits.bytes

instance (u limits : Usage) : Decidable (bounded u limits) := by
  unfold bounded
  infer_instance

inductive State where
  | open (usage : Usage)
  | exhausted
  deriving DecidableEq, Repr

-- Expiry is an observed clock input, not a theorem about wall-clock scheduling.
def admit (limits : Usage) (state : State) (cost : Usage) (expired : Bool) : State :=
  match state with
  | .exhausted => .exhausted
  | .open usage =>
    if expired then .exhausted
    else if bounded (add usage cost) limits then .open (add usage cost)
    else .exhausted

def safe (limits : Usage) : State → Prop
  | .exhausted => True
  | .open usage => bounded usage limits

theorem admission_preserves_bounds (limits usage cost : Usage) (expired : Bool) :
    safe limits (admit limits (.open usage) cost expired) := by
  cases expired with
  | true => simp [admit, safe]
  | false =>
    by_cases h : bounded (add usage cost) limits <;> simp [admit, safe, h]

theorem exhausted_is_terminal (limits cost : Usage) (expired : Bool) :
    admit limits .exhausted cost expired = .exhausted := rfl

def replay (limits : Usage) (state : State) (events : List (Usage × Bool)) : State :=
  events.foldl (fun current event => admit limits current event.1 event.2) state

theorem exhausted_trace_never_succeeds (limits : Usage) (events : List (Usage × Bool)) :
    replay limits .exhausted events = .exhausted := by
  induction events with
  | nil => rfl
  | cons event rest ih => simpa [replay, List.foldl, admit] using ih

theorem replay_preserves_bounds (limits : Usage) (state : State)
    (events : List (Usage × Bool)) (valid : safe limits state) :
    safe limits (replay limits state events) := by
  induction events generalizing state with
  | nil => exact valid
  | cons event rest ih =>
    simp only [replay, List.foldl_cons]
    apply ih
    cases state with
    | exhausted => trivial
    | «open» usage => exact admission_preserves_bounds limits usage event.1 event.2

theorem each_accepted_submission_is_counted (limits usage next : Usage)
    (accepted : admit limits (.open usage) ⟨1, 0, 0⟩ false = .open next) :
    next.commands = usage.commands + 1 := by
  simp only [admit, Bool.false_eq_true, ↓reduceIte] at accepted
  split at accepted
  · cases accepted
    rfl
  · contradiction

-- Two individually fitting components need not fit a request-wide budget.
example : admit ⟨10, 3, 100⟩ (.open ⟨1, 2, 0⟩) ⟨1, 2, 0⟩ false = .exhausted := by decide

-- Payload work can exhaust a budget even after enumeration succeeded.
example : admit ⟨5, 10, 100⟩ (.open ⟨5, 1, 8⟩) ⟨1, 0, 0⟩ false = .exhausted := by decide

structure Observation where
  examined : Nat
  included : Nat
  missing : Nat
  deriving DecidableEq, Repr

def observe (o : Observation) (present : Bool) : Observation :=
  if present then ⟨o.examined + 1, o.included + 1, o.missing⟩
  else ⟨o.examined + 1, o.included, o.missing + 1⟩

theorem missing_is_not_erased (o : Observation) :
    (observe o false).missing = o.missing + 1 := rfl

theorem witnessed_records_are_accounted (o : Observation) (present : Bool)
    (accounted : o.examined = o.included + o.missing) :
    (observe o present).examined = (observe o present).included + (observe o present).missing := by
  cases present <;> simp [observe] <;> omega

inductive Mutation where
  | ready
  | submitted
  | known
  | unknown
  deriving DecidableEq, Repr

def submit : Mutation → Mutation
  | .ready => .submitted
  | state => state

def loseResponse : Mutation → Mutation
  | .submitted => .unknown
  | state => state

theorem unknown_cannot_be_resubmitted : submit (loseResponse (submit .ready)) = .unknown := rfl

end WasModernization.ReadAdmission
