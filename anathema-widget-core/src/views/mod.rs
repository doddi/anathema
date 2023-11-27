pub struct Views {
}

#[cfg(test)]
mod test {
    use crate::testing::view;

    use super::*;

    #[test]
    fn events() {
        let v = view("a-view");
    }
}
