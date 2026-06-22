use loco_rs::testing::prelude::*;
use serial_test::serial;
use trust_ledger_server::app::App;

#[tokio::test]
#[serial]
async fn can_get_ussds() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request
            .post("/api/ussd")
            .json(&serde_json::json!({
                "session_id": "test-session",
                "phone_number": "+233200000000",
                "input": "",
                "menu_state": "main"
            }))
            .await;
        assert_eq!(res.status_code(), 200);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}
