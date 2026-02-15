//! Inspired by  [this source](https://github.com/betcode-org/betfair/blob/1ece2bf0ffede3a41bf14ba4ea1c7004f25964dd/betfairlightweight/streaming/cache.py)

use betfair_adapter::betfair_types::price::Price;
use betfair_adapter::betfair_types::size::Size;
use betfair_stream_types::response::{Position, UpdateSet2, UpdateSet3};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

extern crate alloc;
use alloc::vec::Vec;

/// Data structure to hold prices/traded amount
/// Uses a sorted Vec for better cache locality on small collections (typically 5-20 entries)
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Available<T: UpdateSet> {
    pub book: Vec<(T::Key, T::Value)>,
}

impl<T: UpdateSet> Available<T> {
    pub fn new<A: AsRef<[T]>>(prices: A) -> Self {
        let mut book: Vec<(T::Key, T::Value)> = prices
            .as_ref()
            .iter()
            .filter(|p| !p.should_be_deleted())
            .map(|p| (p.key(), p.value()))
            .collect();
        book.sort_unstable_by(|a, b| a.0.cmp(&b.0));
        Self { book }
    }

    pub fn update<A: AsRef<[T]>>(&mut self, book_update: A) {
        for prices in book_update.as_ref() {
            let key = prices.key(); // this is either `price` or `position`

            match self.book.binary_search_by(|entry| entry.0.cmp(&key)) {
                Ok(idx) => {
                    // Key exists - either update or delete
                    if prices.should_be_deleted() {
                        self.book.remove(idx);
                    } else {
                        self.book[idx].1 = prices.value();
                    }
                }
                Err(idx) => {
                    // Key doesn't exist - insert if not a deletion
                    if !prices.should_be_deleted() {
                        self.book.insert(idx, (key, prices.value()));
                    }
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.book.clear();
    }
}

/// Helper trait fro working with fields that have either 2 or 3 tuple elements (present in the
/// Stream API ladder)
pub trait UpdateSet {
    type Key: core::hash::Hash + PartialEq + Eq + Ord + Serialize + DeserializeOwned;
    type Value: PartialEq + Serialize + DeserializeOwned;
    fn value(&self) -> Self::Value;
    fn key(&self) -> Self::Key;
    fn should_be_deleted(&self) -> bool;
}

impl UpdateSet for UpdateSet2 {
    type Key = Price;
    type Value = Size;

    fn value(&self) -> Self::Value {
        self.1
    }

    fn key(&self) -> Self::Key {
        self.0
    }

    fn should_be_deleted(&self) -> bool {
        self.1 == Size::zero()
    }
}

impl UpdateSet for UpdateSet3 {
    type Key = Position;
    type Value = (Price, Size);

    fn value(&self) -> Self::Value {
        (self.1, self.2)
    }

    fn key(&self) -> Self::Key {
        self.0
    }

    fn should_be_deleted(&self) -> bool {
        self.2 == Size::zero()
    }
}

#[cfg(test)]
mod tests {
    use betfair_adapter::betfair_types::{num, num_u8};
    use pretty_assertions::assert_eq;

    use super::*;

    fn setup_set3() -> Available<UpdateSet3> {
        let prices = &[
            UpdateSet3(
                Position(num_u8!(1)),
                Price::new(num!(1.02)).unwrap(),
                Size::new(num!(34.45)),
            ),
            UpdateSet3(
                Position(num_u8!(0)),
                Price::new(num!(1.01)).unwrap(),
                Size::new(num!(12)),
            ),
        ];
        Available::new(prices)
    }

    #[test]
    fn test_init() {
        let init = setup_set3();

        let expected = vec![
            (
                Position(num_u8!(0)),
                (Price::new(num!(1.01)).unwrap(), Size::new(num!(12))),
            ),
            (
                Position(num_u8!(1)),
                (Price::new(num!(1.02)).unwrap(), Size::new(num!(34.45))),
            ),
        ];

        assert_eq!(init.book, expected);
    }

    #[test]
    fn test_init_2() {
        let prices = &[
            UpdateSet2(Price::new(num!(27)).unwrap(), Size::new(num!(0.95))),
            UpdateSet2(Price::new(num!(13)).unwrap(), Size::new(num!(28.01))),
            UpdateSet2(Price::new(num!(1.02)).unwrap(), Size::new(num!(1157.21))),
        ];
        let init = Available::new(prices);

        let expected = vec![
            (Price::new(num!(1.02)).unwrap(), Size::new(num!(1157.21))),
            (Price::new(num!(13)).unwrap(), Size::new(num!(28.01))),
            (Price::new(num!(27)).unwrap(), Size::new(num!(0.95))),
        ];

        assert_eq!(init.book, expected);
    }

    #[test]
    fn test_clear() {
        let mut init = setup_set3();
        init.clear();

        assert_eq!(init.book, Vec::new());
    }

    #[test]
    fn test_update_set_2() {
        let init = Available::new([
            UpdateSet2(Price::new(num!(27)).unwrap(), Size::new(num!(0.95))),
            UpdateSet2(Price::new(num!(13)).unwrap(), Size::new(num!(28.01))),
            UpdateSet2(Price::new(num!(1.02)).unwrap(), Size::new(num!(1157.21))),
        ]);
        let update = &[UpdateSet2(
            Price::new(num!(27)).unwrap(),
            Size::new(num!(2)),
        )];
        let expected = vec![
            (Price::new(num!(1.02)).unwrap(), Size::new(num!(1157.21))),
            (Price::new(num!(13)).unwrap(), Size::new(num!(28.01))),
            (Price::new(num!(27)).unwrap(), Size::new(num!(2))),
        ];

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected);
    }

    #[test]
    fn test_update_set_3() {
        let init = Available::new([
            UpdateSet3(
                Position(num_u8!(1)),
                Price::new(num!(1.02)).unwrap(),
                Size::new(num!(34.45)),
            ),
            UpdateSet3(
                Position(num_u8!(0)),
                Price::new(num!(1.01)).unwrap(),
                Size::new(num!(12)),
            ),
        ]);
        let update = &[UpdateSet3(
            Position(num_u8!(1)),
            Price::new(num!(1.02)).unwrap(),
            Size::new(num!(22)),
        )];
        let expected = vec![
            (
                Position(num_u8!(0)),
                (Price::new(num!(1.01)).unwrap(), Size::new(num!(12))),
            ),
            (
                Position(num_u8!(1)),
                (Price::new(num!(1.02)).unwrap(), Size::new(num!(22))),
            ),
        ];

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected);
    }

    #[test]
    fn test_update_set_2_delete() {
        let init = Available::new([
            UpdateSet2(Price::new(num!(27)).unwrap(), Size::new(num!(0.95))),
            UpdateSet2(Price::new(num!(13)).unwrap(), Size::new(num!(28.01))),
        ]);
        let update = &[UpdateSet2(
            Price::new(num!(27)).unwrap(),
            Size::new(num!(0)),
        )];
        let expected = vec![(Price::new(num!(13)).unwrap(), Size::new(num!(28.01)))];

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected);
    }

    #[test]
    fn test_update_set_3_delete() {
        let init = Available::new([
            UpdateSet3(
                Position(num_u8!(1)),
                Price::new(num!(1.02)).unwrap(),
                Size::new(num!(34.45)),
            ),
            UpdateSet3(
                Position(num_u8!(0)),
                Price::new(num!(1.01)).unwrap(),
                Size::new(num!(12)),
            ),
        ]);
        let update = &[UpdateSet3(
            Position(num_u8!(1)),
            Price::new(num!(1.02)).unwrap(),
            Size::new(num!(0)),
        )];
        let expected = vec![(
            Position(num_u8!(0)),
            (Price::new(num!(1.01)).unwrap(), Size::new(num!(12))),
        )];

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected);
    }
}
