use anyhow::Result;
use dialoguer::MultiSelect;

pub fn select_services() -> Result<Vec<String>> {
    // TODO: shall be constant or enum?
    let services = vec![
        "cal",
        "dub",
        "openpanel",
        "resend",
        "trigger",
        "sentry",
        "upstash",
    ];
    let selections = MultiSelect::new()
        .with_prompt("Select services to add") // TODO style
        .items(&services)
        .interact()
        .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(selections
        .into_iter()
        .map(|i| services[i].to_string())
        .collect())
}
