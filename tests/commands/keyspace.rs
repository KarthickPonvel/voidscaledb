use crate::common::connect;

#[test]
fn ttl_non_existent() {
    let mut con = connect();

    let res: i64 = redis::cmd("TTL")
        .arg("non_existent_key")
        .query(&mut con)
        .unwrap();
    assert_eq!(res, -2);
}

#[test]
fn ttl_no_expiry() {
    let mut con = connect();
    let k = "test_key";

    redis::cmd("SET")
        .arg(k)
        .arg("v")
        .query::<()>(&mut con)
        .unwrap();

    let res: i64 = redis::cmd("TTL").arg(k).query(&mut con).unwrap();
    assert_eq!(res, -1);
}

#[test]
fn ttl_basic() {
    let mut con = connect();
    let k = "ttl_test";

    redis::cmd("SET")
        .arg(k)
        .arg("v")
        .arg("EX")
        .arg(10)
        .query::<()>(&mut con)
        .unwrap();

    let res: i64 = redis::cmd("TTL").arg(k).query(&mut con).unwrap();
    assert!(res > 0 && res <= 10);
}
