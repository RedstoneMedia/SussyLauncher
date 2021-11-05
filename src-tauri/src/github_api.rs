use reqwest::Client;
use serde_json::Value;

pub async fn make_github_api_request(client : &reqwest::Client, route : String) -> Result<serde_json::Value, String> {
    let response = match client.get(format!("https://api.github.com/{}", route)).send().await {
        Ok(r) => r,
        Err(e) => return Err(e.to_string())
    };
    if !response.status().is_success() {
        return Err(format!("Request returned status code {} with content : {}", response.status().as_u16(), response.text().await.unwrap()))
    }
    match response.json::<serde_json::Value>().await {
        Ok(j) => Ok(j),
        Err(e) => Err(e.to_string())
    }
}

pub fn get_assets(newest_release: &Value) -> Vec<Value> {
    let mut assets = newest_release.get("assets").unwrap().as_array().unwrap().clone();
    // Only keep assets that are .dll or .zip files
    assets.retain(|v| match v.get("content_type").unwrap().as_str().unwrap() {
        "application/x-msdownload" => true,
        "application/x-zip-compressed" => true,
        "application/zip" => true,
        _ => false
    });
    // Sort by download count
    assets.sort_by(|a, b| {
        let a_download_count = a.get("download_count").unwrap().as_u64().unwrap();
        let b_download_count = b.get("download_count").unwrap().as_u64().unwrap();
        b_download_count.cmp(&a_download_count)
    });
    assets
}

pub async fn get_newest_release(client: &Client, username: &String, repository_name: &String) -> Result<Value, String> {
    // Get releases
    let releases_value = make_github_api_request(&client, format!("repos/{}/{}/releases", username, repository_name)).await?;
    let releases = releases_value.as_array().unwrap();
    let newest_release = match releases.iter().find(|v| v.get("draft").unwrap().as_bool().unwrap() == false) {
        Some(r) => r,
        None => return Err("No releases were found".to_string())
    };
    Ok(newest_release.clone())
}
