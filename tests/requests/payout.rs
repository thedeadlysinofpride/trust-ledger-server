use loco_rs::testing::prelude::*;
use serial_test::serial;
use trust_ledger_server::app::App;

#[tokio::test]
#[serial]
async fn can_get_payouts() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.post("/api/groups/2147483647/payout").await;
        assert_eq!(res.status_code(), 404);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}
