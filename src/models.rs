use serde::Serialize;

use num::bigint::BigInt;
use num::rational::{BigRational, Ratio};
use num::FromPrimitive;
use num_traits::cast::ToPrimitive;
use num_traits::identities::{One, Zero};
#[derive(Serialize, Debug, Clone)]
pub struct Item {
    title: String,
    unit: String,
    price: f64,
}

impl Item {
    pub fn new<T: Into<String>, U: Into<String>>(title: T, unit: U, price: f64) -> Item {
        Item {
            title: title.into(),
            unit: unit.into(),
            price,
        }
    }

    pub fn price(&self) -> f64 {
        self.price
    }
}

#[derive(Serialize, Debug)]
pub struct Category {
    title: String,
    items: Vec<Item>,
}

impl Category {
    // Creates a new category
    pub fn new<T: Into<String>>(title: T) -> Category {
        Category {
            title: title.into(),
            items: vec![],
        }
    }

    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub fn items(&self) -> &[Item] {
        &self.items
    }
}
#[derive(Serialize, Debug)]
pub struct WeeklyBasketOffer {
    #[allow(dead_code)]
    categories: Vec<Category>,
}

impl WeeklyBasketOffer {
    pub fn new(categories: Vec<Category>) -> WeeklyBasketOffer {
        WeeklyBasketOffer { categories }
    }

    pub fn categories(&self) -> &[Category] {
        &self.categories
    }
}

#[derive(Debug, Serialize)]
pub struct OrderItem<'a> {
    ref_item: &'a Item,
    quantity: u32,
    sub_price: f64,
}

impl<'a> OrderItem<'a> {
    pub fn new(item: &'a Item, quantity: u32) -> OrderItem<'a> {
        let price =
            BigRational::from_f64(item.price()).unwrap() * BigRational::from_u32(quantity).unwrap();

        let float_result = price.numer().to_f64().unwrap() / price.denom().to_f64().unwrap();

        OrderItem {
            ref_item: item,
            quantity,
            sub_price: float_result,
        }
    }
}
#[derive(Debug, Serialize)]
pub struct OrderPreview<'a> {
    order_items: Vec<OrderItem<'a>>,
    total: f64,
}

impl<'a> OrderPreview<'a> {
    pub fn new(items: Vec<OrderItem<'a>>) -> OrderPreview<'a> {
        let total = items
            .iter()
            .map({
                |item| {
                    Ratio::from_f64(item.ref_item.price()).unwrap()
                        * Ratio::from_u32(item.quantity).unwrap()
                }
            })
            .fold(BigRational::zero(), |acc, x| acc + x);

        let float_result = total.numer().to_f64().unwrap() / total.denom().to_f64().unwrap();

        OrderPreview {
            order_items: items,
            total: float_result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Category, Item, WeeklyBasketOffer};
    use chrono::Utc;
    #[test]
    fn create_some_categories() {
        // Fruits
        let mut fruits = Category::new("fruits");

        let item = Item::new("Fraise", "250 gr", 1.0);
        fruits.add_item(item);

        assert_eq!(1, fruits.items.len());

        let offer = WeeklyBasketOffer::new(vec![fruits]);
        assert_eq!(1, offer.categories.len());
    }
}
