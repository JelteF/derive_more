% What #[derive(Add)] generates

# Tuple structs

When deriving `Add` for a tuple struct with two fields like this:

```
#[derive(Add)]
struct MyInts(i32, i32)
```

Code like this will be generated:

```
impl ::std::ops::Add for MyInts {
    type Output = MyInts;
    fn add(self, rhs: MyInts) -> MyInts {
        MyInts(self.0.add(rhs.0), self.1.add(rhs.1))
    }
}
```

The behaviour is similar with more or less fields.



# Regular structs

When deriving `Add` for a regular struct with two fields like this:

```
#[derive(Add)]
struct Point2D {
    x: i32,
    y: i32,
}
```

Code like this will be generated:

```
impl ::std::ops::Add for Point2D {
    type Output = Point2D;
    fn add(self, rhs: Point2D) -> Point2D {
        Point2D {
            x: self.x.add(rhs.x),
            y: self.y.add(rhs.y),
        }
    }
}
```

The behaviour is similar for more or less fields.


# Enums

There's a big difference between the code that is generated for the two struct
types and the one that is generated for enums. The code for enums returns
`Result<EnumType>` instead of an `EnumType` itself. This is because adding an
enum to another enum is only possible if both are the same variant. This makes
the generated code much more complex as well, because this check needs to be
done. For instance when deriving `Add` for an enum like this:

```
#[derive(Add)]
enum MixedInts {
    SmallInt(i32),
    BigInt(i64),
    TwoSmallInts(i32, i32),
    NamedSmallInts { x: i32, y: i32 },
    UnsignedOne(u32),
    UnsignedTwo(u32),
    Unit,
}
```

Code like this will be generated:

```
impl ::std::ops::Add for MixedInts {
    type Output = Result<MixedInts, &'static str>;
    fn add(self, rhs: MixedInts) -> Result<MixedInts, &'static str> {
        match (self, rhs) {
            (MixedInts::SmallInt(__l_0), MixedInts::SmallInt(__r_0)) => {
                Ok(MixedInts::SmallInt(__l_0.add(__r_0)))
            }
            (MixedInts::BigInt(__l_0), MixedInts::BigInt(__r_0)) => {
                Ok(MixedInts::BigInt(__l_0.add(__r_0)))
            }
            (MixedInts::TwoSmallInts(__l_0, __l_1), MixedInts::TwoSmallInts(__r_0, __r_1)) => {
                Ok(MixedInts::TwoSmallInts(__l_0.add(__r_0), __l_1.add(__r_1)))
            }
            (MixedInts::NamedSmallInts { x: __l_0, y: __l_1 },
             MixedInts::NamedSmallInts { x: __r_0, y: __r_1 }) => {
                Ok(MixedInts::NamedSmallInts {
                    x: __l_0.add(__r_0),
                    y: __l_1.add(__r_1),
                })
            }
            (MixedInts::UnsignedOne(__l_0), MixedInts::UnsignedOne(__r_0)) => {
                Ok(MixedInts::UnsignedOne(__l_0.add(__r_0)))
            }
            (MixedInts::UnsignedTwo(__l_0), MixedInts::UnsignedTwo(__r_0)) => {
                Ok(MixedInts::UnsignedTwo(__l_0.add(__r_0)))
            }
            (MixedInts::Unit, MixedInts::Unit) => Err("Cannot add() unit variants"),
            _ => Err("Trying to add mismatched enum variants"),
        }
    }
}
```

Also note the Unit type that throws an error when adding it to itself.
