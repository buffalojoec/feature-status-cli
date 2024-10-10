use {
    serde::{Deserialize, Deserializer, Serialize},
    solana_sdk::pubkey::Pubkey,
};

fn deserialize_pubkey<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Pubkey::try_from(s.as_str()).map_err(serde::de::Error::custom)
}

#[derive(PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FeatureStatus {
    Active,
    Inactive,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feature {
    #[serde(deserialize_with = "deserialize_pubkey")]
    pub id: Pubkey,
    pub description: String,
    pub status: FeatureStatus,
    pub since_slot: Option<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureSet {
    pub software_versions: Vec<String>,
    pub feature_set: u64,
    pub stake_percent: f64,
    pub rpc_percent: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterFeatureSets {
    pub tool_feature_set: u64,
    pub feature_sets: Vec<FeatureSet>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoftwareVersion {
    pub software_version: String,
    pub stake_percent: f64,
    pub rpc_percent: f64,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClusterSoftwareVersions {
    pub tool_software_version: String,
    pub software_versions: Vec<SoftwareVersion>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureStatusReport {
    pub features: Vec<Feature>,
    pub feature_activation_allowed: bool,
    pub cluster_feature_sets: ClusterFeatureSets,
    pub cluster_software_versions: ClusterSoftwareVersions,
}

impl FeatureStatusReport {
    pub fn is_active(&self, feature_id: &Pubkey) -> bool {
        self.features
            .iter()
            .any(|feature| &feature.id == feature_id && feature.status == FeatureStatus::Active)
    }
    pub fn from_json_file(json_file: &str) -> serde_json::Result<Self> {
        let file = std::fs::File::open(json_file).expect("Unable to open file");
        let reader = std::io::BufReader::new(file);
        serde_json::from_reader(reader)
    }
}
