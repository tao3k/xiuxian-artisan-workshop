import Std.Tactic

namespace WasModernization

inductive KeyEnumeration where
  | blockingKeys
  | cursorScan
  deriving DecidableEq, Repr

inductive OrgAuthority where
  | wendao
  | asp
  deriving DecidableEq, Repr

structure SnapshotArchitecture where
  metadataCopies : Nat
  independentReadRounds : Nat
  keyEnumeration : KeyEnumeration
  scanResultsDeduplicated : Bool
  orgAuthority : OrgAuthority
  deriving DecidableEq, Repr

def legacyArchitecture : SnapshotArchitecture where
  metadataCopies := 1
  independentReadRounds := 5
  keyEnumeration := .blockingKeys
  scanResultsDeduplicated := false
  orgAuthority := .wendao

def modernArchitecture : SnapshotArchitecture where
  metadataCopies := 0
  independentReadRounds := 1
  keyEnumeration := .cursorScan
  scanResultsDeduplicated := true
  orgAuthority := .asp

def isIncrementalAndDuplicateSafe (architecture : SnapshotArchitecture) : Prop :=
  architecture.keyEnumeration = .cursorScan ∧ architecture.scanResultsDeduplicated = true

def satisfiesHotPathPolicy (architecture : SnapshotArchitecture) : Prop :=
  architecture.metadataCopies = 0 ∧
    architecture.independentReadRounds = 1 ∧
    isIncrementalAndDuplicateSafe architecture ∧
    architecture.orgAuthority = .asp

theorem legacyArchitectureViolatesPolicy : ¬satisfiesHotPathPolicy legacyArchitecture := by
  simp [satisfiesHotPathPolicy, legacyArchitecture]

theorem modernArchitectureSatisfiesPolicy : satisfiesHotPathPolicy modernArchitecture := by
  simp [satisfiesHotPathPolicy, isIncrementalAndDuplicateSafe, modernArchitecture]

theorem wendaoCannotOwnModernOrgAuthority
    (architecture : SnapshotArchitecture)
    (policy : satisfiesHotPathPolicy architecture) :
    architecture.orgAuthority ≠ .wendao := by
  simp [satisfiesHotPathPolicy] at policy
  simp [policy.2.2.2]

def clonedMetadataBytes (candidates fieldsPerCandidate bytesPerValue : Nat) : Nat :=
  candidates * fieldsPerCandidate * bytesPerValue

def borrowedMetadataBytes (_candidates _fieldsPerCandidate _bytesPerValue : Nat) : Nat := 0

theorem borrowingEliminatesMetadataCloneBytes (candidates fieldsPerCandidate bytesPerValue : Nat) :
    borrowedMetadataBytes candidates fieldsPerCandidate bytesPerValue = 0 := by
  rfl

theorem cloningHasPositiveCost
    (candidates fieldsPerCandidate bytesPerValue : Nat)
    (hc : 0 < candidates)
    (hf : 0 < fieldsPerCandidate)
    (hb : 0 < bytesPerValue) :
    0 < clonedMetadataBytes candidates fieldsPerCandidate bytesPerValue := by
  simp only [clonedMetadataBytes]
  exact Nat.mul_pos (Nat.mul_pos hc hf) hb

def serialReadLatency (rounds latencyPerRound : Nat) : Nat := rounds * latencyPerRound

def parallelReadLatency (latencyPerRound : Nat) : Nat := latencyPerRound

theorem fiveIndependentReadsDoNotRegressLatency (latencyPerRound : Nat) :
    parallelReadLatency latencyPerRound ≤ serialReadLatency 5 latencyPerRound := by
  simp [parallelReadLatency, serialReadLatency]
  omega

theorem fiveIndependentReadsStrictlyImproveLatency
    (latencyPerRound : Nat)
    (positiveLatency : 0 < latencyPerRound) :
    parallelReadLatency latencyPerRound < serialReadLatency 5 latencyPerRound := by
  simp [parallelReadLatency, serialReadLatency]
  omega

-- Request-work model: coverage reuses one description, removing two reads.
theorem sharedIndexDescriptionSavesTwoReads (descriptionCost otherWork : Nat) :
    otherWork + 3 * descriptionCost =
      (otherWork + descriptionCost) + 2 * descriptionCost := by
  omega

-- COUNT does not constrain the size of an observed protocol reply.
structure ScanReply where
  countHint : Nat
  returnedKeys : Nat
  nextCursor : Nat
  deriving Repr

theorem countHintIsNotAReplyBound (hint : Nat) :
    ∃ reply : ScanReply, reply.countHint = hint ∧ hint < reply.returnedKeys := by
  exact ⟨⟨hint, hint + 1, 0⟩, rfl, Nat.lt_succ_self hint⟩

-- Arithmetic admission abstraction; admitKey below also models duplicates.
def admitsRetainedKeys (retained incoming budget : Nat) : Prop :=
  retained + incoming ≤ budget

theorem admittedKeysRespectBudget (retained incoming budget : Nat)
    (admitted : admitsRetainedKeys retained incoming budget) :
    retained + incoming ≤ budget := by
  exact admitted

theorem emptyPageDoesNotImplyCompletion :
    ∃ reply : ScanReply, reply.returnedKeys = 0 ∧ reply.nextCursor ≠ 0 := by
  exact ⟨⟨256, 0, 1⟩, rfl, by decide⟩

theorem blockingKeysHasNoFixedWorkBound :
    ∀ budget : Nat, ∃ keyCount : Nat, budget < keyCount := by
  intro budget
  exact ⟨budget + 1, Nat.lt_succ_self budget⟩

structure ScanState where
  keys : Nat
  bytes : Nat
  calls : Nat

def withinBudget (state limits : ScanState) : Prop :=
  state.keys ≤ limits.keys ∧ state.bytes ≤ limits.bytes ∧ state.calls ≤ limits.calls

def admitKey (state limits : ScanState) (size : Nat) (duplicate : Bool) : Option ScanState :=
  if duplicate then some state
  else if state.keys + 1 ≤ limits.keys ∧ state.bytes + size ≤ limits.bytes then
    some { state with keys := state.keys + 1, bytes := state.bytes + size }
  else none

theorem retainedAdmissionPreservesBudget (state limits next : ScanState) (size : Nat)
    (duplicate : Bool) (valid : withinBudget state limits)
    (accepted : admitKey state limits size duplicate = some next) :
    withinBudget next limits := by
  unfold admitKey at accepted
  split at accepted
  · cases accepted
    exact valid
  · split at accepted
    · cases accepted
      rename_i bounds
      exact ⟨bounds.1, bounds.2, valid.2.2⟩
    · contradiction

inductive CommandKind where
  | read
  | mutation
  deriving DecidableEq

def maxSubmissions : CommandKind → Nat
  | .read => 2
  | .mutation => 1

theorem mutationIsNeverAutomaticallyReplayed : maxSubmissions .mutation = 1 := rfl

def invalidateGeneration (current failed : Nat) : Option Nat :=
  if current = failed then none else some current

theorem staleFailurePreservesGeneration (current failed : Nat) (different : current ≠ failed) :
    invalidateGeneration current failed = some current := by
  simp [invalidateGeneration, different]

end WasModernization
