pub(crate) mod characters;
mod container;
pub mod display_object;
pub mod library;
pub mod tag_utils;

mod test {

    #[test]
    fn it_works() {
        use std::collections::BTreeMap;
        use std::ops::Bound::{Excluded, Unbounded};

        let mut map = BTreeMap::new();
        map.insert(3, "a");
        map.insert(5, "b");
        map.insert(8, "c");
        let next = map
            .range((Excluded(3), Unbounded))
            .map(|(_, v)| *v)
            .next()
            .unwrap();
        assert_eq!(next, "b");
    }
}
