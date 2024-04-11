/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * Copyright 2024 MNX Cloud, Inc.
 */

use serde::Deserialize;
use serde_json::{Error, Value};
use smartos_shared::serde_helpers::{
    deserialize_into_bool, deserialize_into_u64,
};

#[derive(Deserialize, Debug)]
struct TestStructU64 {
    #[serde(deserialize_with = "deserialize_into_u64")]
    pub value: u64,
}

#[test]
fn test_deserialize_into_u64() {
    let zero: TestStructU64 =
        serde_json::from_str("{\"value\":0}").expect("failed to parse 0");
    assert!(zero.value == 0);

    let one: TestStructU64 =
        serde_json::from_str("{\"value\":1}").expect("failed to parse 1");
    assert!(one.value == 1);

    let negative_one: TestStructU64 =
        serde_json::from_str("{\"value\":-1}").expect("failed to parse -1");
    assert!(negative_one.value == 0);

    let less_than_one_float: TestStructU64 =
        serde_json::from_str("{\"value\":0.001}")
            .expect("failed to parse 0.001");
    assert!(less_than_one_float.value == 0);

    let float: TestStructU64 =
        serde_json::from_str("{\"value\":1.0}").expect("failed to parse 1.0");
    assert!(float.value == 1);

    let negative_float: TestStructU64 =
        serde_json::from_str("{\"value\":-1.0}").expect("failed to parse -1.0");
    assert!(negative_float.value == 0);

    let string_zero: TestStructU64 =
        serde_json::from_str("{\"value\":\"0\"}").expect("failed to parse '0'");
    assert!(string_zero.value == 0);

    let string_one: TestStructU64 =
        serde_json::from_str("{\"value\":\"1\"}").expect("failed to parse '1'");
    assert!(string_one.value == 1);

    let string_negative_one: TestStructU64 =
        serde_json::from_str("{\"value\":\"-1\"}")
            .expect("failed to parse '-1'");
    assert!(string_negative_one.value == 0);

    let string_less_than_one_float: TestStructU64 =
        serde_json::from_str("{\"value\":\"0.01\"}")
            .expect("failed to parse '0.01'");
    assert!(string_less_than_one_float.value == 0);

    let string_float: TestStructU64 =
        serde_json::from_str("{\"value\":\"1.0\"}")
            .expect("failed to parse '1.0'");
    assert!(string_float.value == 1);

    let bogus: TestStructU64 = serde_json::from_str("{\"value\":\"NaN\"}")
        .expect("failed to parse string");
    assert!(bogus.value == 0);

    let bogus2: TestStructU64 = serde_json::from_str("{\"value\":\"hi\"}")
        .expect("failed to parse string");
    assert!(bogus2.value == 0);

    let bogus3: TestStructU64 = serde_json::from_str("{\"value\":\"\"}")
        .expect("failed to parse string");
    assert!(bogus3.value == 0);

    // Any other type else should return an Error
    let bool_type: Result<TestStructU64, Error> =
        serde_json::from_str("{\"value\":false}");
    assert!(bool_type.is_err());

    let array_type: Result<TestStructU64, Error> =
        serde_json::from_str("{\"value\":[]}");
    assert!(array_type.is_err());

    let object_type: Result<TestStructU64, Error> =
        serde_json::from_str("{\"value\":{}}");
    assert!(object_type.is_err());

    let null_type: Result<TestStructU64, Error> =
        serde_json::from_str("{\"value\":null}");
    assert!(null_type.is_err());
}

#[derive(Deserialize, Debug)]
struct TestStructBool {
    #[serde(deserialize_with = "deserialize_into_bool")]
    pub value: bool,
}

#[test]
fn test_deserialize_into_bool() {
    let bool_true: TestStructBool =
        serde_json::from_str("{\"value\":true}").expect("failed to parse true");
    assert!(bool_true.value == true);

    let string_true: TestStructBool =
        serde_json::from_str("{\"value\":\"true\"}")
            .expect("failed to parse 'true'");
    assert!(string_true.value == true);

    let string_true2: TestStructBool =
        serde_json::from_str("{\"value\":\"TRUE\"}")
            .expect("failed to parse 'TRUE'");
    assert!(string_true2.value == true);

    let string_yes: TestStructBool =
        serde_json::from_str("{\"value\":\"yes\"}")
            .expect("failed to parse 'yes'");
    assert!(string_yes.value == true);

    let bool_false: TestStructBool = serde_json::from_str("{\"value\":false}")
        .expect("failed to parse false");
    assert!(bool_false.value == false);

    let string_false: TestStructBool =
        serde_json::from_str("{\"value\":\"false\"}")
            .expect("failed to parse 'false'");
    assert!(string_false.value == false);

    let string_no: TestStructBool = serde_json::from_str("{\"value\":\"no\"}")
        .expect("failed to parse 'no'");
    assert!(string_no.value == false);

    let string_empty: TestStructBool =
        serde_json::from_str("{\"value\":\"\"}").expect("failed to parse ''");
    assert!(string_empty.value == false);

    // Any other type else should return an Error
    let number_type: Result<TestStructBool, Error> =
        serde_json::from_str("{\"value\":1}");
    assert!(number_type.is_err());

    let null_type: Result<TestStructBool, Error> =
        serde_json::from_str("{\"value\":null}");
    assert!(null_type.is_err());

    let object_type: Result<TestStructBool, Error> =
        serde_json::from_str("{\"value\":{}}");
    assert!(object_type.is_err());

    let array_type: Result<TestStructBool, Error> =
        serde_json::from_str("{\"value\":[]}");
    assert!(array_type.is_err());
}
