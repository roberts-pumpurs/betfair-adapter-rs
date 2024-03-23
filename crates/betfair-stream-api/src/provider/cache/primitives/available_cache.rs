//! Inspired by https://github.com/betcode-org/betfair/blob/1ece2bf0ffede3a41bf14ba4ea1c7004f25964dd/betfairlightweight/streaming/cache.py

use std::collections::BTreeMap;

use betfair_adapter::betfair_types::price::Price;
use betfair_adapter::betfair_types::size::Size;
use betfair_adapter::rust_decimal::Decimal;
use betfair_stream_types::response::{Position, UpdateSet2, UpdateSet3};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Data structure to hold prices/traded amount
#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize, Deserialize, Eq, Hash, Ord)]
pub struct Available<T: UpdateSet> {
    book: BTreeMap<T::Key, T::Value>,
}

impl<T: UpdateSet> Available<T> {
    pub fn new(prices: impl AsRef<[T]>) -> Self {
        let mut instance = Self {
            book: BTreeMap::new(),
        };

        instance.update(prices);
        instance
    }

    pub fn update(&mut self, book_update: impl AsRef<[T]>) {
        for prices in book_update.as_ref() {
            let key = prices.key(); // this is either `price` or `position`
            let value = prices.value(); // this is either `(price, size)` or `size`

            // If the "key" is zero, then we need to delete the item
            if prices.should_be_deleted() {
                self.book.remove(&key);
            } else {
                self.book.insert(key, value);
            }
        }
    }

    pub fn clear(&mut self) {
        self.book.clear();
    }
}

pub trait UpdateSet {
    type Key: std::hash::Hash + PartialEq + Eq + Ord + Serialize + DeserializeOwned;
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
        self.0.clone()
    }

    fn should_be_deleted(&self) -> bool {
        self.1 == Size::new(Decimal::ZERO)
    }
}

impl UpdateSet for UpdateSet3 {
    type Key = Position;
    type Value = (Price, Size);

    fn value(&self) -> Self::Value {
        (self.1.clone(), self.2)
    }

    fn key(&self) -> Self::Key {
        self.0.clone()
    }

    fn should_be_deleted(&self) -> bool {
        self.2 == Size::new(Decimal::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use rust_decimal_macros::dec;

    use super::*;

    fn setup_set3() -> Available<UpdateSet3> {
        let prices = &[
            UpdateSet3(
                Position(dec!(1)),
                Price::new(dec!(1.02)).unwrap(),
                Size::new(dec!(34.45)),
            ),
            UpdateSet3(
                Position(dec!(0)),
                Price::new(dec!(1.01)).unwrap(),
                Size::new(dec!(12)),
            ),
        ];
        Available::new(prices)
    }

    #[test]
    fn test_init() {
        let init = setup_set3();

        let mut expected = BTreeMap::new();
        expected.insert(
            Position(dec!(0)),
            (Price::new(dec!(1.01)).unwrap(), Size::new(dec!(12))),
        );
        expected.insert(
            Position(dec!(1)),
            (Price::new(dec!(1.02)).unwrap(), Size::new(dec!(34.45))),
        );

        assert_eq!(init.book, expected)
    }

    #[test]
    fn test_init_2() {
        let prices = &[
            UpdateSet2(Price::new(dec!(27)).unwrap(), Size::new(dec!(0.95))),
            UpdateSet2(Price::new(dec!(13)).unwrap(), Size::new(dec!(28.01))),
            UpdateSet2(Price::new(dec!(1.02)).unwrap(), Size::new(dec!(1157.21))),
        ];
        let init = Available::new(prices);

        let mut expected = BTreeMap::new();
        expected.insert(Price::new(dec!(1.02)).unwrap(), Size::new(dec!(1157.21)));
        expected.insert(Price::new(dec!(13)).unwrap(), Size::new(dec!(28.01)));
        expected.insert(Price::new(dec!(27)).unwrap(), Size::new(dec!(0.95)));

        assert_eq!(init.book, expected)
    }

    #[test]
    fn test_clear() {
        let mut init = setup_set3();
        init.clear();

        assert_eq!(init.book, BTreeMap::new())
    }

    #[test]
    fn test_update_set_2() {
        let init = Available::new([
            UpdateSet2(Price::new(dec!(27)).unwrap(), Size::new(dec!(0.95))),
            UpdateSet2(Price::new(dec!(13)).unwrap(), Size::new(dec!(28.01))),
            UpdateSet2(Price::new(dec!(1.02)).unwrap(), Size::new(dec!(1157.21))),
        ]);
        let update = &[UpdateSet2(
            Price::new(dec!(27)).unwrap(),
            Size::new(dec!(2)),
        )];
        let mut expected = BTreeMap::new();
        expected.insert(Price::new(dec!(1.02)).unwrap(), Size::new(dec!(1157.21)));
        expected.insert(Price::new(dec!(13)).unwrap(), Size::new(dec!(28.01)));
        expected.insert(Price::new(dec!(27)).unwrap(), Size::new(dec!(2)));

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected)
    }

    #[test]
    fn test_update_set_3() {
        let init = Available::new([
            UpdateSet3(
                Position(dec!(1)),
                Price::new(dec!(1.02)).unwrap(),
                Size::new(dec!(34.45)),
            ),
            UpdateSet3(
                Position(dec!(0)),
                Price::new(dec!(1.01)).unwrap(),
                Size::new(dec!(12)),
            ),
        ]);
        let update = &[UpdateSet3(
            Position(dec!(1)),
            Price::new(dec!(1.02)).unwrap(),
            Size::new(dec!(22)),
        )];
        let mut expected = BTreeMap::new();
        expected.insert(
            Position(dec!(1)),
            (Price::new(dec!(1.02)).unwrap(), Size::new(dec!(22))),
        );
        expected.insert(
            Position(dec!(0)),
            (Price::new(dec!(1.01)).unwrap(), Size::new(dec!(12))),
        );

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected)
    }

    #[test]
    fn test_update_set_2_delete() {
        let init = Available::new([
            UpdateSet2(Price::new(dec!(27)).unwrap(), Size::new(dec!(0.95))),
            UpdateSet2(Price::new(dec!(13)).unwrap(), Size::new(dec!(28.01))),
        ]);
        let update = &[UpdateSet2(
            Price::new(dec!(27)).unwrap(),
            Size::new(dec!(0)),
        )];
        let mut expected = BTreeMap::new();
        expected.insert(Price::new(dec!(13)).unwrap(), Size::new(dec!(28.01)));

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected)
    }

    #[test]
    fn test_update_set_3_delete() {
        let init = Available::new([
            UpdateSet3(
                Position(dec!(1)),
                Price::new(dec!(1.02)).unwrap(),
                Size::new(dec!(34.45)),
            ),
            UpdateSet3(
                Position(dec!(0)),
                Price::new(dec!(1.01)).unwrap(),
                Size::new(dec!(12)),
            ),
        ]);
        let update = &[UpdateSet3(
            Position(dec!(1)),
            Price::new(dec!(1.02)).unwrap(),
            Size::new(dec!(0)),
        )];
        let mut expected = BTreeMap::new();
        expected.insert(
            Position(dec!(0)),
            (Price::new(dec!(1.01)).unwrap(), Size::new(dec!(12))),
        );

        let mut actual = init;
        actual.update(update);

        assert_eq!(actual.book, expected)
    }
}
