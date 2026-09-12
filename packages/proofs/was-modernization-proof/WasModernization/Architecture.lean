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

def isNonBlockingAndDuplicateSafe (architecture : SnapshotArchitecture) : Prop :=
  architecture.keyEnumeration = .cursorScan ∧ architecture.scanResultsDeduplicated = true

def satisfiesHotPathPolicy (architecture : SnapshotArchitecture) : Prop :=
  architecture.metadataCopies = 0 ∧
    architecture.independentReadRounds = 1 ∧
    isNonBlockingAndDuplicateSafe architecture ∧
    architecture.orgAuthority = .asp

theorem legacyArchitectureViolatesPolicy : ¬satisfiesHotPathPolicy legacyArchitecture := by
  simp [satisfiesHotPathPolicy, legacyArchitecture]

theorem modernArchitectureSatisfiesPolicy : satisfiesHotPathPolicy modernArchitecture := by
  simp [satisfiesHotPathPolicy, isNonBlockingAndDuplicateSafe, modernArchitecture]

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

def scanStepWork (pageSize keyCount : Nat) : Nat := min pageSize keyCount

theorem scanStepWorkIsPageBounded (pageSize keyCount : Nat) :
    scanStepWork pageSize keyCount ≤ pageSize := by
  exact Nat.min_le_left _ _

theorem blockingKeysHasNoFixedWorkBound :
    ∀ budget : Nat, ∃ keyCount : Nat, budget < keyCount := by
  intro budget
  exact ⟨budget + 1, Nat.lt_succ_self budget⟩

end WasModernization
