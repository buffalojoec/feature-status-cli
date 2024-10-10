use {
    crate::{status_report::FeatureStatusReport, Network},
    regex::Regex,
    solana_sdk::pubkey::Pubkey,
    std::{error::Error, fs::File, io::Write},
};

async fn fetch_agave_features(
    agave_version: &str,
) -> Result<Vec<(String, Pubkey)>, Box<dyn Error>> {
    let url = format!(
        "https://raw.githubusercontent.com/anza-xyz/agave/refs/tags/v{}/sdk/src/feature_set.rs",
        agave_version
    );
    let response = reqwest::get(url).await?.text().await?;

    let module_regex =
        Regex::new(r#"pub mod (\w+)\s*\{\s*solana_sdk::declare_id!\(\"([a-zA-Z0-9]+)\"\);"#)?;

    Ok(module_regex
        .captures_iter(&response)
        .map(|cap| (cap[1].to_string(), Pubkey::try_from(&cap[2]).unwrap()))
        .collect::<Vec<_>>())
}

pub async fn emit(
    agave_version: &str,
    reports: &[(Network, FeatureStatusReport)],
    out_file: &str,
) -> std::io::Result<()> {
    let agave_features = fetch_agave_features(agave_version).await.unwrap();

    // Agave feature activations:
    //  1. testnet
    //  2. devnet
    //  3. mainnet-beta
    let mut inactive = Vec::new();
    let mut active_testnet = Vec::new();
    let mut active_devnet = Vec::new();
    let mut active_mainnet = Vec::new();

    for (label, id) in agave_features {
        let mut is_active_testnet = false;
        let mut is_active_devnet = false;
        let mut is_active_mainnet = false;

        for (network, report) in reports {
            match network {
                Network::Devnet => is_active_devnet = report.is_active(&id),
                Network::Testnet => is_active_testnet = report.is_active(&id),
                Network::MainnetBeta => is_active_mainnet = report.is_active(&id),
            }
        }

        let label_with_id = format!("{}::id()", label);

        if is_active_mainnet {
            active_mainnet.push(label_with_id);
        } else if is_active_devnet {
            active_devnet.push(label_with_id);
        } else if is_active_testnet {
            active_testnet.push(label_with_id);
        } else {
            inactive.push(label_with_id);
        }
    }

    let emit_contents = format!(
        r#"
// List of agave supported feature flags.
// As of `{}`.
static AGAVE_FEATURES: &[Pubkey] = &[
    // Inactive on all clusters.
    {}
    // Active on testnet.
    {}
    // Active on devnet.
    {}
    // Active on mainnet-beta.
    {}
];
"#,
        agave_version,
        inactive.join(",\n    "),
        active_testnet.join(",\n    "),
        active_devnet.join(",\n    "),
        active_mainnet.join(",\n    "),
    );

    let mut file = File::create(out_file)?;
    file.write_all(emit_contents.as_bytes())?;

    Ok(())
}
