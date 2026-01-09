#![cfg_attr(not(feature = "std"), no_std)]

mod structs {
    use derive_more::Default;

    #[test]
    fn tuple_struct() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        struct TupleStruct(i32, Option<NonDefault>);

        assert_eq!(TupleStruct::default(), TupleStruct(0, None))
    }

    #[test]
    fn named_fields_struct() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        struct NamedFieldsStruct {
            field1: i32,
            field2: Option<NonDefault>,
        }

        assert_eq!(
            NamedFieldsStruct::default(),
            NamedFieldsStruct {
                field1: 0,
                field2: None
            }
        )
    }

    #[test]
    fn tuple_struct_with_default_attr() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        struct TupleStruct(
            #[default(42)] i32,
            #[default(Some(NonDefault))] Option<NonDefault>,
            #[default(Self::default_field3())] i32,
        );
        impl TupleStruct {
            fn default_field3() -> i32 {
                73
            }
        }

        assert_eq!(
            TupleStruct::default(),
            TupleStruct(42, Some(NonDefault), 73)
        )
    }

    #[test]
    fn named_fields_struct_with_default_attr() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        struct NamedFieldsStruct {
            #[default(42)]
            field1: i32,
            #[default(Some(NonDefault))]
            field2: Option<NonDefault>,
            #[default(Self::default_field3())]
            field3: i32,
        }
        impl NamedFieldsStruct {
            fn default_field3() -> i32 {
                73
            }
        }

        assert_eq!(
            NamedFieldsStruct::default(),
            NamedFieldsStruct {
                field1: 42,
                field2: Some(NonDefault),
                field3: 73,
            }
        )
    }

    #[test]
    fn generic_tuple_struct() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        struct GenericTupleStruct<T>(u32, Option<T>);

        assert_eq!(
            GenericTupleStruct::<NonDefault>::default(),
            GenericTupleStruct(0, None)
        )
    }

    #[test]
    fn generic_named_fields_struct() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        struct GenericNamedFieldsStruct<T> {
            field1: u32,
            field2: Option<T>,
        }

        assert_eq!(
            GenericNamedFieldsStruct::<NonDefault>::default(),
            GenericNamedFieldsStruct {
                field1: 0,
                field2: None
            }
        )
    }

    #[test]
    fn generic_manual_bounds() {
        trait MyTrait {
            type Assoc;
        }

        #[derive(PartialEq, Debug)]
        struct NonDefault;

        impl MyTrait for NonDefault {
            type Assoc = u32;
        }

        #[derive(Default, PartialEq, Debug)]
        #[default(bound(T::Assoc: Default))]
        struct GenericWithBound<T: MyTrait> {
            field: T::Assoc,
        }

        assert_eq!(
            GenericWithBound::<NonDefault>::default(),
            GenericWithBound::<NonDefault> { field: 0 }
        )
    }
}

mod enums {
    use derive_more::Default;
    #[test]
    fn unit_variants() {
        #[derive(Default, PartialEq, Debug)]
        enum UnitVariants {
            #[allow(unused)]
            Variant1,
            #[default]
            Variant2,
            #[allow(unused)]
            Variant3,
        }
        assert_eq!(UnitVariants::default(), UnitVariants::Variant2)
    }

    #[test]
    fn tuple_variants() {
        #[derive(Default, PartialEq, Debug)]
        enum TupleVariants {
            #[allow(unused)]
            Variant1(i32),
            #[default]
            Variant2(Option<i32>),
            #[allow(unused)]
            Variant3,
        }
        assert_eq!(TupleVariants::default(), TupleVariants::Variant2(None))
    }

    #[test]
    fn fields_variants() {
        #[derive(Default, PartialEq, Debug)]
        enum FieldsVariants {
            #[allow(unused)]
            Variant1 { field: u32 },
            #[default]
            Variant2 { field: Option<u32> },
            #[allow(unused)]
            Variant3,
        }

        assert_eq!(
            FieldsVariants::default(),
            FieldsVariants::Variant2 { field: None }
        )
    }

    #[test]
    fn tuple_variants_with_default_attr() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        enum TupleVariantsWithDefault {
            #[allow(unused)]
            Variant1(i32),
            #[default]
            Variant2(
                #[default(42)] i32,
                #[default(Some(NonDefault))] Option<NonDefault>,
                #[default(Self::default_field())] i32,
            ),
            #[allow(unused)]
            Variant3,
        }
        impl TupleVariantsWithDefault {
            fn default_field() -> i32 {
                73
            }
        }

        assert_eq!(
            TupleVariantsWithDefault::default(),
            TupleVariantsWithDefault::Variant2(42, Some(NonDefault), 73)
        )
    }

    #[test]
    fn fields_variants_with_default_attr() {
        #[derive(PartialEq, Debug)]
        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        enum FieldsVariantsWithDefault {
            #[allow(unused)]
            Variant1 { field: u32 },
            #[default]
            Variant2 {
                #[default(42)]
                field1: i32,
                #[default(Some(NonDefault))]
                field2: Option<NonDefault>,
                #[default(Self::default_field())]
                field3: i32,
            },
            #[allow(unused)]
            Variant3,
        }
        impl FieldsVariantsWithDefault {
            fn default_field() -> i32 {
                73
            }
        }

        assert_eq!(
            FieldsVariantsWithDefault::default(),
            FieldsVariantsWithDefault::Variant2 {
                field1: 42,
                field2: Some(NonDefault),
                field3: 73,
            }
        )
    }

    #[test]
    fn generic_tuple_variants() {
        #[derive(PartialEq, Debug)]

        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        enum GenericTupleVariants<T> {
            #[allow(unused)]
            Variant1(T),
            #[default]
            Variant2(Option<T>),
            #[allow(unused)]
            Variant3,
        }

        assert_eq!(
            GenericTupleVariants::<NonDefault>::default(),
            GenericTupleVariants::Variant2(None)
        )
    }

    #[test]
    fn generic_fields_variants() {
        #[derive(PartialEq, Debug)]

        struct NonDefault;
        #[derive(Default, PartialEq, Debug)]
        enum GenericFieldsVariants<T> {
            #[allow(unused)]
            Variant1 { field: T },
            #[default]
            Variant2 { field: Option<T> },
            #[allow(unused)]
            Variant3,
        }
        assert_eq!(
            GenericFieldsVariants::<NonDefault>::default(),
            GenericFieldsVariants::Variant2 { field: None }
        )
    }

    #[test]
    fn generic_manual_bounds_enum() {
        trait MyTrait {
            type Assoc;
        }

        #[derive(PartialEq, Debug)]
        struct NonDefault;

        impl MyTrait for NonDefault {
            type Assoc = u32;
        }

        #[derive(Default, Debug, PartialEq)]
        #[default(bound(T::Assoc: Default))]
        enum GenericEnumWithBound<T: MyTrait> {
            #[default]
            Variant { field: T::Assoc },
            #[allow(unused)]
            Other,
        }

        assert_eq!(
            GenericEnumWithBound::<NonDefault>::default(),
            GenericEnumWithBound::<NonDefault>::Variant { field: 0 }
        );
    }
}
