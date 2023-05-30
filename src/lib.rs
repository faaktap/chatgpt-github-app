use dotenv::dotenv;
use flowsnet_platform_sdk::write_error_log;
use github_flows::{
    get_octo, listen_to_event,
    octocrab::{models::events::payload::EventPayload, models::events::payload::IssuesEventAction},
};
use openai_flows::{chat_completion, ChatModel, ChatOptions};
use std::env;

#[no_mangle]
#[tokio::main(flavor = "current_thread")]
pub async fn run() {
    dotenv().ok();

    let login: String = match env::var("login") {
        Err(_) => "alabulei1".to_string(),
        Ok(name) => name,
    };

    let owner: String = match env::var("owner") {
        Err(_) => "second-state".to_string(),
        Ok(name) => name,
Expand All
	@@ -27,29 +32,41 @@ pub async fn run() {
        Ok(name) => name,
    };

    listen_to_event(
        &login,
        &owner,
        &repo,
        vec!["issue_comment", "issues"],
        |payload| handler(&owner, &repo, &openai_key_name, payload),
    )
    .await;
}

async fn handler(owner: &str, repo: &str, openai_key_name: &str, payload: EventPayload) {
    let octo = get_octo(Some(String::from(owner)));
    let issues = octo.issues(owner, repo);

    match payload {
        EventPayload::IssueCommentEvent(e) => {
            let last_comment_id = store_flows::get("last_created_comment").unwrap_or_default();
            if e.comment.id.into_inner() != last_comment_id.as_u64().unwrap_or_default() {
                if let Some(b) = e.comment.body {
                    if let Some(r) = chat_completion(
                        openai_key_name,
                        &format!("issue#{}", e.issue.number),
                        &b,
                        &ChatOptions::default(),
                    ) {
                        match issues.create_comment(e.issue.number, r.choice).await {
                            Ok(comment) => {
                                store_flows::set(
                                    "last_created_comment",
                                    serde_json::to_value(comment.id.into_inner()).unwrap(),
                                );
                            }
                            Err(e) => {
                                write_error_log!(e.to_string());
                            }
                        }
                    }
                }
Expand All
	@@ -69,7 +86,7 @@ async fn handler(owner: &str, repo: &str, openai_key_name: &str, payload: EventP
            let co = ChatOptions {
                model: ChatModel::GPT4,
                restart: true,
                restarted_sentence: Some(&prompt),
            };

            if let Some(r) = chat_completion(
Expand All
	@@ -78,8 +95,16 @@ async fn handler(owner: &str, repo: &str, openai_key_name: &str, payload: EventP
                &prompt,
                &co,
            ) {
                match issues.create_comment(e.issue.number, r.choice).await {
                    Ok(comment) => {
                        store_flows::set(
                            "last_created_comment",
                            serde_json::to_value(comment.id.into_inner()).unwrap(),
                        );
                    }
                    Err(e) => {
                        write_error_log!(e.to_string());
                    }
                }
            }
        }
