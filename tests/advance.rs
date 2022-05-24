use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::json;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_simple_scenario() {
    let mock_server = MockServer::start().await;
    let contributor_path = format!("{}/search/contributor/test", &mock_server.uri());

    Mock::given(method("GET"))
        .and(path("/search/repositories"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            {
                "total_count": 1,
                "items": [
                {
                    "stargazers_count": 199,
                    "contributors_url": contributor_path,
                    "full_name": "test",
                    "other": {}

                }
            ],
                "other": {}
            }
        )))
        .expect(1)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/search/contributor/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!(
            [
                {
                    "login": "test_user",
                    "contributions": 199,
                    "other": {}
                }
            ]
        )))
        .expect(1)
        .mount(&mock_server)
        .await;

    let url = format!("{}/search/repositories", &mock_server.uri());

    let cli = Command::cargo_bin("bus_factor")
        .unwrap()
        .env("GITHUB_ACCESS_TOKEN", "PAT_TOKEN")
        .args(&["-l", "rust"])
        .args(&["-p", "1"])
        .args(&["-g", &url])
        .assert();

    cli.success().stdout(
        predicate::str::contains("project: test")
            .and(predicate::str::contains("user: test_user"))
            .and(predicate::str::contains("percentage: 100")),
    );
}
