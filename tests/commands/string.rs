use crate::common::connect;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn key(name: &str) -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    format!("t:{}:{}", name, ts)
}

fn set(con: &mut redis::Connection, key: &str, value: &str) {
    redis::cmd("SET")
        .arg(key)
        .arg(value)
        .query::<()>(con)
        .unwrap();
}

fn set_cmd(
    con: &mut redis::Connection,
    key: &str,
    value: &str,
    args: &[&str],
) -> redis::RedisResult<Option<String>> {
    let mut cmd = redis::cmd("SET");
    cmd.arg(key).arg(value);
    for a in args {
        cmd.arg(*a);
    }
    cmd.query(con)
}

fn get(con: &mut redis::Connection, key: &str) -> Option<String> {
    redis::cmd("GET").arg(key).query(con).unwrap()
}

fn ttl(con: &mut redis::Connection, key: &str) -> i64 {
    redis::cmd("TTL").arg(key).query(con).unwrap()
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

#[test]
fn core_get_set() {
    let mut con = connect();
    let k = key("core");

    assert_eq!(get(&mut con, &k), None);
    set(&mut con, &k, "a");
    assert_eq!(get(&mut con, &k), Some("a".into()));
}

#[test]
fn overwrite() {
    let mut con = connect();
    let k = key("ovr");

    set(&mut con, &k, "a");
    set(&mut con, &k, "b");
    set(&mut con, &k, "c");

    assert_eq!(get(&mut con, &k), Some("c".into()));
}

#[test]
fn ex_basic() {
    let mut con = connect();
    let k = key("ex");

    redis::cmd("SET")
        .arg(&k)
        .arg("v")
        .arg("EX")
        .arg(1)
        .query::<()>(&mut con)
        .unwrap();

    assert_eq!(get(&mut con, &k), Some("v".into()));

    thread::sleep(Duration::from_secs(2));

    assert_eq!(get(&mut con, &k), None);
}

#[test]
fn px_basic() {
    let mut con = connect();
    let k = key("px");

    redis::cmd("SET")
        .arg(&k)
        .arg("v")
        .arg("PX")
        .arg(100)
        .query::<()>(&mut con)
        .unwrap();

    thread::sleep(Duration::from_millis(200));

    assert_eq!(get(&mut con, &k), None);
}

#[test]
fn exat_future_and_past() {
    let mut con = connect();
    let k1 = key("exat_f");
    let k2 = key("exat_p");

    let future = now_secs() + 5;
    let past = now_secs() - 5;

    redis::cmd("SET")
        .arg(&k1)
        .arg("v")
        .arg("EXAT")
        .arg(future as i64)
        .query::<()>(&mut con)
        .unwrap();

    redis::cmd("SET")
        .arg(&k2)
        .arg("v")
        .arg("EXAT")
        .arg(past as i64)
        .query::<()>(&mut con)
        .unwrap();

    assert_eq!(get(&mut con, &k2), None);
}

#[test]
fn pxat_future_and_past() {
    let mut con = connect();
    let k1 = key("pxat_f");
    let k2 = key("pxat_p");

    let future = now_millis() + 5000;
    let past = now_millis().saturating_sub(1000);

    redis::cmd("SET")
        .arg(&k1)
        .arg("v")
        .arg("PXAT")
        .arg(future as i64)
        .query::<()>(&mut con)
        .unwrap();

    redis::cmd("SET")
        .arg(&k2)
        .arg("v")
        .arg("PXAT")
        .arg(past as i64)
        .query::<()>(&mut con)
        .unwrap();

    assert_eq!(get(&mut con, &k2), None);
}

#[test]
fn nx_basic() {
    let mut con = connect();
    let k = key("nx");

    assert_eq!(
        set_cmd(&mut con, &k, "a", &["NX"]).unwrap(),
        Some("OK".into())
    );

    assert_eq!(set_cmd(&mut con, &k, "b", &["NX"]).unwrap(), None);
}

#[test]
fn xx_basic() {
    let mut con = connect();
    let k = key("xx");

    assert_eq!(set_cmd(&mut con, &k, "a", &["XX"]).unwrap(), None);

    set(&mut con, &k, "a");

    assert_eq!(
        set_cmd(&mut con, &k, "b", &["XX"]).unwrap(),
        Some("OK".into())
    );
}

#[test]
fn get_flag() {
    let mut con = connect();
    let k = key("get");

    set(&mut con, &k, "a");

    let old: String = redis::cmd("SET")
        .arg(&k)
        .arg("b")
        .arg("GET")
        .query(&mut con)
        .unwrap();

    assert_eq!(old, "a");
}

#[test]
fn nx_get() {
    let mut con = connect();
    let k = key("nxget");

    set(&mut con, &k, "a");

    let old: Option<String> = redis::cmd("SET")
        .arg(&k)
        .arg("b")
        .arg("NX")
        .arg("GET")
        .query(&mut con)
        .unwrap();

    assert_eq!(old, Some("a".into()));
}

#[test]
fn keepttl_preserve() {
    let mut con = connect();
    let k = key("kt");

    redis::cmd("SET")
        .arg(&k)
        .arg("a")
        .arg("EX")
        .arg(10)
        .query::<()>(&mut con)
        .unwrap();

    let t1 = ttl(&mut con, &k);

    redis::cmd("SET")
        .arg(&k)
        .arg("b")
        .arg("KEEPTTL")
        .query::<()>(&mut con)
        .unwrap();

    let t2 = ttl(&mut con, &k);

    assert_eq!(t1, t2);
}

#[test]
fn keepttl_no_reset() {
    let mut con = connect();
    let k = key("kt2");

    redis::cmd("SET")
        .arg(&k)
        .arg("a")
        .arg("EX")
        .arg(1)
        .query::<()>(&mut con)
        .unwrap();

    let t1 = ttl(&mut con, &k);

    redis::cmd("SET")
        .arg(&k)
        .arg("b")
        .arg("KEEPTTL")
        .query::<()>(&mut con)
        .unwrap();

    let t2 = ttl(&mut con, &k);

    assert_eq!(t1, t2);

    thread::sleep(Duration::from_secs(2));

    assert_eq!(get(&mut con, &k), None);
}

#[test]
fn del_single_key() {
    let mut con = connect();
    let k = key("del1");

    set(&mut con, &k, "a");

    let res: i64 = redis::cmd("DEL").arg(&k).query(&mut con).unwrap();

    assert_eq!(res, 1);
    assert_eq!(get(&mut con, &k), None);
}

#[test]
fn del_missing_key() {
    let mut con = connect();
    let k = key("del_missing");

    let res: i64 = redis::cmd("DEL").arg(&k).query(&mut con).unwrap();

    assert_eq!(res, 0);
}

#[test]
fn del_multiple_keys() {
    let mut con = connect();
    let k1 = key("del_m1");
    let k2 = key("del_m2");

    set(&mut con, &k1, "a");
    set(&mut con, &k2, "b");

    let res: i64 = redis::cmd("DEL").arg(&k1).arg(&k2).query(&mut con).unwrap();

    assert_eq!(res, 2);

    assert_eq!(get(&mut con, &k1), None);
    assert_eq!(get(&mut con, &k2), None);
}

#[test]
fn del_after_expiry() {
    let mut con = connect();
    let k = key("del_exp");

    redis::cmd("SET")
        .arg(&k)
        .arg("v")
        .arg("EX")
        .arg(1)
        .query::<()>(&mut con)
        .unwrap();

    thread::sleep(Duration::from_secs(2));

    let res: i64 = redis::cmd("DEL").arg(&k).query(&mut con).unwrap();

    assert_eq!(res, 0);
}

#[test]
fn syntax_error() {
    let mut con = connect();

    let res: redis::RedisResult<String> = redis::cmd("SET")
        .arg("k")
        .arg("v")
        .arg("EX")
        .query(&mut con);

    assert!(res.is_err());
}

#[test]
fn incr_basic() {
    let mut con = connect();
    let k = key("incr");

    let r1: i64 = redis::cmd("INCR").arg(&k).query(&mut con).unwrap();
    assert_eq!(r1, 1);

    let r2: i64 = redis::cmd("INCR").arg(&k).query(&mut con).unwrap();
    assert_eq!(r2, 2);

    assert_eq!(get(&mut con, &k), Some("2".into()));
}

#[test]
fn decr_basic() {
    let mut con = connect();
    let k = key("decr");

    let r1: i64 = redis::cmd("DECR").arg(&k).query(&mut con).unwrap();
    assert_eq!(r1, -1);

    let r2: i64 = redis::cmd("DECR").arg(&k).query(&mut con).unwrap();
    assert_eq!(r2, -2);

    assert_eq!(get(&mut con, &k), Some("-2".into()));
}

#[test]
fn incrby_decrby() {
    let mut con = connect();
    let k = key("by");

    let r1: i64 = redis::cmd("INCRBY")
        .arg(&k)
        .arg(10)
        .query(&mut con)
        .unwrap();
    assert_eq!(r1, 10);

    let r2: i64 = redis::cmd("DECRBY").arg(&k).arg(3).query(&mut con).unwrap();
    assert_eq!(r2, 7);

    assert_eq!(get(&mut con, &k), Some("7".into()));
}

#[test]
fn incr_decr_errors() {
    let mut con = connect();
    let k = key("err");

    set(&mut con, &k, "not_an_int");

    let r1: redis::RedisResult<i64> = redis::cmd("INCR").arg(&k).query(&mut con);
    assert!(r1.is_err());

    let r2: redis::RedisResult<i64> = redis::cmd("DECR").arg(&k).query(&mut con);
    assert!(r2.is_err());
}

#[test]
fn incr_decr_on_expired_key() {
    let mut con = connect();
    let k = key("exp");

    redis::cmd("SET")
        .arg(&k)
        .arg("10")
        .arg("EX")
        .arg(1)
        .query::<()>(&mut con)
        .unwrap();

    thread::sleep(Duration::from_secs(2));

    let r: i64 = redis::cmd("INCR").arg(&k).query(&mut con).unwrap();
    assert_eq!(r, 1);
}

#[test]
fn incr_bounds_overflow() {
    let mut con = connect();
    let k = key("overflow");

    set(&mut con, &k, &i64::MAX.to_string());

    let r: redis::RedisResult<i64> = redis::cmd("INCR").arg(&k).query(&mut con);
    assert!(r.is_err());
}
