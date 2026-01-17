use betfair_stream_types::response::*;
#[test]
fn fixture_streaming_503() {
    let data = std::fs::read_to_string("./tests/resources/streaming_503.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_connection() {
    let data = std::fs::read_to_string("./tests/resources/streaming_connection.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_market_definition() {
    let data =
        std::fs::read_to_string("./tests/resources/streaming_market_definition.json").unwrap();
    let _res = serde_json::from_str::<market_change_message::MarketDefinition>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_heartbeat() {
    let data = std::fs::read_to_string("./tests/resources/streaming_mcm_HEARTBEAT.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_resub_delta() {
    let data = std::fs::read_to_string("./tests/resources/streaming_mcm_RESUB_DELTA.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_sub_image_no_market_def() {
    let data =
        std::fs::read_to_string("./tests/resources/streaming_mcm_SUB_IMAGE_no_market_def.json")
            .unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_sub_image() {
    let data = std::fs::read_to_string("./tests/resources/streaming_mcm_SUB_IMAGE.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_update_md() {
    let data = std::fs::read_to_string("./tests/resources/streaming_mcm_UPDATE_md.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_update_tv() {
    let data = std::fs::read_to_string("./tests/resources/streaming_mcm_UPDATE_tv.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_mcm_update() {
    let data = std::fs::read_to_string("./tests/resources/streaming_mcm_update.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_ocm_empty_image() {
    let data = std::fs::read_to_string("./tests/resources/streaming_ocm_EMPTY_IMAGE.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_ocm_full_image() {
    let data = std::fs::read_to_string("./tests/resources/streaming_ocm_FULL_IMAGE.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}
#[test]
fn fixture_streaming_ocm_full_image_missing_rc() {
    let data =
        std::fs::read_to_string("./tests/resources/streaming_ocm_FULL_IMAGE_MISSING_RC.json")
            .unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_ocm_new_full_image() {
    let data =
        std::fs::read_to_string("./tests/resources/streaming_ocm_NEW_FULL_IMAGE.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_ocm_sub_image() {
    let data = std::fs::read_to_string("./tests/resources/streaming_ocm_SUB_IMAGE.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_ocm_update() {
    let data = std::fs::read_to_string("./tests/resources/streaming_ocm_UPDATE.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}

#[test]
fn fixture_streaming_status() {
    let data = std::fs::read_to_string("./tests/resources/streaming_status.json").unwrap();
    let _res = serde_json::from_str::<ResponseMessage>(&data).unwrap();
}
