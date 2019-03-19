#[allow(dead_code)]
struct MyStruct {
    source: String,
}

trait MyTrait {
    fn my_trait(source: String) -> Self;

    fn description(&self) -> String;
}

impl MyTrait for MyStruct {
    fn my_trait(source: String) -> Self {
        // Self ≡ MyStruct
        MyStruct { source }
    }

    fn description(&self) -> String {
        String::from("MyStruct(...)")
    }
}

impl MyTrait for String {
    fn my_trait(source: String) -> Self {
        // Self ≡ String
        source // Identity
    }

    fn description(&self) -> String {
        self.clone() // Borrowing ⇒ Ownership
    }
}

#[cfg(test)]
mod test {
    use crate::MyStruct;
    use crate::MyTrait;

    #[test]
    fn should_create_mystruct_1() {
        let data: MyStruct = MyTrait::my_trait(String::from("test"));
        let description = data.description(); // "MyStruct(...)"

        assert_eq!(description, String::from("MyStruct(...)"))
    }

    #[test]
    fn should_create_mystruct_2() {
        let data = <MyStruct>::my_trait(String::from("test"));
        let description = data.description(); // "MyStruct(...)"

        assert_eq!(description, String::from("MyStruct(...)"))
    }

    #[test]
    fn should_create_mystruct_3() {
        let data = MyStruct {
            source: String::from("test"),
        };
        let description = data.description(); // "MyStruct(...)"

        assert_eq!(description, String::from("MyStruct(...)"))
    }

    #[test]
    fn should_create_string_1() {
        let data: String = MyTrait::my_trait(String::from("test"));
        let description = data.description(); // "test"

        assert_eq!(description, String::from("test"))
    }

    #[test]
    fn should_create_string_2() {
        let data = <String>::my_trait(String::from("test"));
        let description = data.description(); // "test"

        assert_eq!(description, String::from("test"))
    }

    #[test]
    fn should_create_string_3() {
        let data = String::from("test");
        let description = data.description(); // "test"

        assert_eq!(description, String::from("test"))
    }
}
