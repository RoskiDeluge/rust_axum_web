#![allow(unused)]

use serde_json::json;

use anyhow::Result;

#[tokio::test]
async fn test_quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8080")?;

    hc.do_get("/hello2/Mike").await?.print().await?;

    let req_login = hc.do_post(
        "/api/login",
        json!({
            "username": "demo1",
            "pwd": "welcome"
        }),
    );

    // req_login.await?.print().await?;

    let req_create_ticket = hc.do_post(
        "/api/tickets",
        json!({
            "title": "Ticket AAA"
        }),
    );

    // hc.do_delete("/api/tickets/1").await?.print().await?;

    req_create_ticket.await?.print().await?;

    hc.do_get("/api/tickets").await?.print().await?;

    Ok(())
}
