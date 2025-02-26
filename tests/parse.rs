use indoc::indoc;
use ini::{parse_str, Key, Value};

#[test]
fn should_parse() {
    let input = indoc! {r#"
        anon = 42
        boop = hi
        8 = great
        [general]
        foo = bar
        num=42
        bum    =     42
        a = world 
        b = world ;skip me (mind the gap)

        c = world;skip me
        d = tom foo
        e = 1, 2, 3
        f = one, two, three; junk skip
        [bar junk]
        foo = bar
        num=42
        bum    =     42
        a = world 
        b = world ;skip me (mind the gap)

        c = world;skip me
        d = tom foo
        e = 1, 2, 3
        f = one, two, three; junk skip

        [another]  ; with comment
        foo = bar
        num=42
        bum    =     42
        a = world 
        b = world ;skip me (mind the gap)

        c = world;skip me
        d = tom foo
        e = 1, 2, 3
        f = one, two, three; junk skip

        [empty]

        "#};
    let table = parse_str(input).unwrap();

    // read the anonymous table
    let anon = table.get("_").unwrap();
    assert_eq!(3, anon.len());
    assert_eq!(Some(&Value::Num(42)), anon.get(&Key::Str("anon")));
    assert_eq!(Some(&Value::Str("hi")), anon.get(&"boop".into()));
    assert_eq!(Some(&Value::Str("great")), anon.get(&Key::Num(8)));

    // read the empty table
    let empty = table.get("empty").unwrap();
    assert_eq!(0, empty.len());

    // Read the categories
    for i in ["general", "bar junk", "another"] {
        let map = table.get(i).unwrap();
        assert_eq!(9, map.len());
        assert_eq!(Some(&Value::Str("bar")), map.get(&"foo".into()));
        assert_eq!(Some(&Value::Num(42)), map.get(&"num".into()));
        assert_eq!(Some(&Value::Num(42)), map.get(&"bum".into()));
        assert_eq!(Some(&Value::Str("world")), map.get(&"a".into()));
        assert_eq!(Some(&Value::Str("world")), map.get(&"b".into()));
        assert_eq!(Some(&Value::Str("world")), map.get(&"c".into()));
        assert_eq!(Some(&Value::Str("tom foo")), map.get(&"d".into()));
        assert_eq!(
            Some(&Value::Array(vec![
                Value::Num(1),
                Value::Num(2),
                Value::Num(3)
            ])),
            map.get(&"e".into())
        );
        assert_eq!(
            Some(&Value::Array(vec![
                Value::Str("one"),
                Value::Str("two"),
                Value::Str("three")
            ])),
            map.get(&"f".into())
        );
    }
}
