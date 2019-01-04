#![allow(dead_code)]

trait MyTrait {
    fn new(source: String) -> Self;

    fn description(&self) -> String;
}

struct MyStruct {
    source: String
}

impl MyTrait for MyStruct {
    fn new(source: String) -> Self {
        MyStruct { source }
    }

    fn description(&self) -> String {
        "MyStruct(...)".to_string()
    }

}

impl MyTrait for String {
    fn new(source: String) -> Self {
        source
    }

    fn description(&self) -> String {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::MyStruct;
    use crate::MyTrait;

    #[test]
    fn it_implements_with_a_struct() {
        let object : MyStruct = MyTrait::new("test".to_string());

        assert_eq!(object.description(), "MyStruct(...)".to_string());
    }

    #[test]
    fn it_implements_with_a_string() {
        let object : String = MyTrait::new("test".to_string());

        assert_eq!(object.description(), "test".to_string());
    }
}
