use serde::{Deserialize, Serialize};

/// Entity type enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    #[serde(rename = "PERSON")]
    /// A human individual.
    Person,
    #[serde(rename = "ORGANIZATION")]
    /// A company, team, or institution.
    Organization,
    #[serde(rename = "CONCEPT")]
    /// An abstract idea or topic.
    #[default]
    Concept,
    #[serde(rename = "PROJECT")]
    /// A project, repository, or initiative.
    Project,
    #[serde(rename = "TOOL")]
    /// A software tool or library.
    Tool,
    #[serde(rename = "SKILL")]
    /// A reusable capability or skill.
    Skill,
    #[serde(rename = "LOCATION")]
    /// A physical or logical location.
    Location,
    #[serde(rename = "EVENT")]
    /// A time-bounded event.
    Event,
    #[serde(rename = "DOCUMENT")]
    /// A document or note.
    Document,
    #[serde(rename = "CODE")]
    /// A code artifact.
    Code,
    #[serde(rename = "API")]
    /// An API surface.
    Api,
    #[serde(rename = "ERROR")]
    /// An error category or instance.
    Error,
    #[serde(rename = "PATTERN")]
    /// A design or implementation pattern.
    Pattern,
    #[serde(rename = "OTHER")]
    /// A custom entity type represented by free-form text.
    Other(String),
}

impl std::fmt::Display for EntityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityType::Person => write!(f, "PERSON"),
            EntityType::Organization => write!(f, "ORGANIZATION"),
            EntityType::Concept => write!(f, "CONCEPT"),
            EntityType::Project => write!(f, "PROJECT"),
            EntityType::Tool => write!(f, "TOOL"),
            EntityType::Skill => write!(f, "SKILL"),
            EntityType::Location => write!(f, "LOCATION"),
            EntityType::Event => write!(f, "EVENT"),
            EntityType::Document => write!(f, "DOCUMENT"),
            EntityType::Code => write!(f, "CODE"),
            EntityType::Api => write!(f, "API"),
            EntityType::Error => write!(f, "ERROR"),
            EntityType::Pattern => write!(f, "PATTERN"),
            EntityType::Other(s) => write!(f, "OTHER({s})"),
        }
    }
}

/// Relation type enumeration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RelationType {
    #[serde(rename = "WORKS_FOR")]
    /// Employment/affiliation relation.
    WorksFor,
    #[serde(rename = "PART_OF")]
    /// Membership/composition relation.
    PartOf,
    #[serde(rename = "USES")]
    /// Usage/dependency relation.
    Uses,
    #[serde(rename = "DEPENDS_ON")]
    /// Hard dependency relation.
    DependsOn,
    #[serde(rename = "SIMILAR_TO")]
    /// Similarity relation.
    SimilarTo,
    #[serde(rename = "LOCATED_IN")]
    /// Spatial or logical containment relation.
    LocatedIn,
    #[serde(rename = "CREATED_BY")]
    /// Authorship/ownership relation.
    CreatedBy,
    #[serde(rename = "DOCUMENTED_IN")]
    /// Documentation linkage relation.
    DocumentedIn,
    #[serde(rename = "RELATED_TO")]
    /// Generic relatedness relation.
    #[default]
    RelatedTo,
    #[serde(rename = "IMPLEMENTS")]
    /// Implementation relation.
    Implements,
    #[serde(rename = "EXTENDS")]
    /// Inheritance/extension relation.
    Extends,
    #[serde(rename = "CONTAINS")]
    /// Container/content relation.
    Contains,
    #[serde(rename = "REFERENCES")]
    /// Semantic reference relation between skill definitions and target entities.
    References,
    #[serde(rename = "GOVERNS")]
    /// Governing relation between one skill and controlled tools/intents/workflows.
    Governs,
    #[serde(rename = "MANIFESTS")]
    /// Manifestation relation between one skill and persona entities.
    Manifests,
    #[serde(rename = "ATTACHED_TO")]
    /// Attachment relation between one skill and binary/reference assets.
    AttachedTo,
    #[serde(rename = "OTHER")]
    /// A custom relation represented by free-form text.
    Other(String),
}

impl std::fmt::Display for RelationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationType::WorksFor => write!(f, "WORKS_FOR"),
            RelationType::PartOf => write!(f, "PART_OF"),
            RelationType::Uses => write!(f, "USES"),
            RelationType::DependsOn => write!(f, "DEPENDS_ON"),
            RelationType::SimilarTo => write!(f, "SIMILAR_TO"),
            RelationType::LocatedIn => write!(f, "LOCATED_IN"),
            RelationType::CreatedBy => write!(f, "CREATED_BY"),
            RelationType::DocumentedIn => write!(f, "DOCUMENTED_IN"),
            RelationType::RelatedTo => write!(f, "RELATED_TO"),
            RelationType::Implements => write!(f, "IMPLEMENTS"),
            RelationType::Extends => write!(f, "EXTENDS"),
            RelationType::Contains => write!(f, "CONTAINS"),
            RelationType::References => write!(f, "REFERENCES"),
            RelationType::Governs => write!(f, "GOVERNS"),
            RelationType::Manifests => write!(f, "MANIFESTS"),
            RelationType::AttachedTo => write!(f, "ATTACHED_TO"),
            RelationType::Other(s) => write!(f, "OTHER({s})"),
        }
    }
}
