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

#[test]
fn exists_non_existent() {
    let mut con = connect();

    let res: i64 = redis::cmd("EXISTS")
        .arg("non_existent_key")
        .query(&mut con)
        .unwrap();
    assert_eq!(res, 0);
}

#[test]
fn exists_single() {
    let mut con = connect();
    let k = "exists_single_key";

    redis::cmd("SET")
        .arg(k)
        .arg("val")
        .query::<()>(&mut con)
        .unwrap();

    let res: i64 = redis::cmd("EXISTS").arg(k).query(&mut con).unwrap();
    assert_eq!(res, 1);
}

#[test]
fn exists_multiple() {
    let mut con = connect();
    let k1 = "exists_multi_1";
    let k2 = "exists_multi_2";

    redis::cmd("SET")
        .arg(k1)
        .arg("v1")
        .query::<()>(&mut con)
        .unwrap();

    redis::cmd("SET")
        .arg(k2)
        .arg("v2")
        .query::<()>(&mut con)
        .unwrap();

    let res1: i64 = redis::cmd("EXISTS")
        .arg(k1)
        .arg("non_existent_key")
        .arg(k2)
        .query(&mut con)
        .unwrap();
    assert_eq!(res1, 2);

    let res2: i64 = redis::cmd("EXISTS")
        .arg(k1)
        .arg(k1)
        .query(&mut con)
        .unwrap();
    assert_eq!(res2, 2);
}
