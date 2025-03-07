use indoc::indoc;

#[test]
fn should_deserialize() {
    #[derive(serde::Deserialize)]
    struct Nested {
        n0: u8,
        n1: u8,
        n2: Option<u8>,
    }

    #[derive(serde::Deserialize)]
    struct Newtype(u32);

    #[derive(serde::Deserialize)]
    struct TupleStruct<'a>(u8, &'a str);

    #[derive(serde::Deserialize, Debug, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum Enum {
        ThingA,
        ThingB,
        ThingC,
        ThingD,
    }

    #[derive(serde::Deserialize)]
    struct Test<'a> {
        b0: bool,
        b1: bool,
        b2: bool,
        b3: bool,
        b4: bool,
        b5: bool,
        n0: u8,
        n1: u16,
        n2: u32,
        n3: u64,
        n4: u8,
        n5: u16,
        n6: u32,
        n7: u64,
        n8: Vec<u8>,
        c0: char,
        new_type: Newtype,
        #[serde(borrow)]
        t0: (u8, &'a str),
        #[serde(borrow)]
        t1: TupleStruct<'a>,
        nested_none: Nested,
        nested_some: Nested,
        e0: Enum,
        e1: Enum,
        e2: Enum,
        e3: Enum,
        m0: String,
        m1: String,
    }

    #[rustfmt::skip]
    let input = indoc! {r#"
        b0 = true
        b1 = false

        b2 = 1
        b3 = 0
        b4 = true
        b5 = false
        n0 = 0
        n1 = 1
        n2 = 2
        n3 = 3
        n4 = 4
        n5 = 5
        n6 = 6
        n7 = 7
        n8 = 0,1,  2, 3, 4
        c0 = x
        new_type = 42
        t0 = 42, hi
        t1 = 42, hello_world

        e0 = thing_a
        e1 = thing_b
        e2 = thing_c
        e3 = 3

        m0 = \
            hello \
            world

        m1 = \
            this is a paragraph\
            \
            next paragraph

        [nested_none]
        n0 = 42
        n1 = 84

        [nested_some]
        n0 = 42
        n1 = 84
        n2 = 5


        "#};

    // TODO
    //      test unit
    //      test unit struct
    let test: Test = ini::from_str(input).unwrap();
    assert!(test.b0);
    assert!(!test.b1);
    assert!(test.b2);
    assert!(!test.b3);
    assert!(test.b4);
    assert!(!test.b5);
    assert_eq!(0, test.n0);
    assert_eq!(1, test.n1);
    assert_eq!(2, test.n2);
    assert_eq!(3, test.n3);
    assert_eq!(4, test.n4);
    assert_eq!(5, test.n5);
    assert_eq!(6, test.n6);
    assert_eq!(7, test.n7);
    assert_eq!(vec![0, 1, 2, 3, 4], test.n8);
    assert_eq!('x', test.c0);
    assert_eq!(42, test.new_type.0);
    assert_eq!((42, "hi"), test.t0);
    assert_eq!((42, "hello_world"), (test.t1.0, test.t1.1));
    assert_eq!(42, test.nested_none.n0);
    assert_eq!(84, test.nested_none.n1);
    assert_eq!(None, test.nested_none.n2);
    assert_eq!(42, test.nested_some.n0);
    assert_eq!(84, test.nested_some.n1);
    assert_eq!(Some(5), test.nested_some.n2);
    assert_eq!(Enum::ThingA, test.e0);
    assert_eq!(Enum::ThingB, test.e1);
    assert_eq!(Enum::ThingC, test.e2);
    assert_eq!(Enum::ThingD, test.e3);
    assert_eq!("hello world", test.m0);
    assert_eq!("this is a paragraph\nnext paragraph", test.m1);
}
