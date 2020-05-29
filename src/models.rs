use serde::Serialize;

use num::bigint::BigInt;
use num::rational::{BigRational, Ratio};
use num::FromPrimitive;
use num_traits::cast::ToPrimitive;
use num_traits::identities::{One, Zero};

use std::collections::HashMap;
use std::rc::Rc;

/// An Item represents an item within the current
/// week basket.
#[derive(Serialize, Debug, Clone)]
pub struct Item {
    /// The title of the item
    title: String,
    /// The ubuying unit of the item
    unit: String,
    /// The price of the item
    price: f64,
}

impl Item {
    /// Creates a new item with some title unit and price
    pub fn new<T: Into<String>, U: Into<String>>(title: T, unit: U, price: f64) -> Item {
        Item {
            title: title.into(),
            unit: unit.into(),
            price,
        }
    }

    /// Get the price of the item
    pub fn price(&self) -> f64 {
        self.price
    }
}

/// A category grouping some items
#[derive(Serialize, Debug)]
pub struct Category {
    /// The title of the category
    title: String,
    /// The list of items
    items: Vec<Rc<Item>>,
}

impl Category {
    // Creates a new category with a specific title
    pub fn new<T: Into<String>>(title: T) -> Category {
        Category {
            title: title.into(),
            items: vec![],
        }
    }

    /// Add an items to the category
    pub fn add_item(&mut self, item: Item) {
        self.items.push(item);
    }

    /// Retrieves the items of the categories
    pub fn items(&self) -> &[Item] {
        &self.items
    }
}

/// Catalog groups all the categories available
#[derive(Serialize, Debug)]
pub struct Catalog {
    /// The cateogries inside the catalog
    categories: Vec<Category>,
}

impl Catalog {
    /// Creates a new catalog from the categories
    pub fn new(categories: Vec<Category>) -> Catalog {
        Catalog { categories }
    }

    /// Retrieves the categories from the catagalog
    pub fn categories(&self) -> &[Category] {
        &self.categories
    }
}

/// ItemPickup represents a pickup from someone
/// with some article.
#[derive(Debug, Serialize)]
pub struct ItemPickUp {
    ref_item: Rc<Item>,
    quantity: u32,
    sub_price: f64,
}

impl ItemPickUp {
    /// Creates a new `PickUp` with some quantity
    pub fn new(item: &Rc<Item>, quantity: u32) -> ItemPickUp {
        let price =
            BigRational::from_f64(item.price()).unwrap() * BigRational::from_u32(quantity).unwrap();

        let float_result = price.numer().to_f64().unwrap() / price.denom().to_f64().unwrap();

        ItemPickUp {
            ref_item: item.clone(),
            quantity,
            sub_price: float_result,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Cart {
    pick_ups: Vec<ItemPickUp>,
    total: f64,
}

impl Cart {
    pub fn new(items: Vec<ItemPickUp>) -> Cart {
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

        Cart {
            pick_ups: items,
            total: float_result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Catalog, Category, Item};
    use chrono::Utc;
    #[test]
    fn create_some_categories() {
        // Fruits
        let mut fruits = Category::new("fruits");

        let item = Item::new("Fraise", "250 gr", 1.0);
        fruits.add_item(item);

        assert_eq!(1, fruits.items.len());

        let offer = Catalog::new(vec![fruits]);
        assert_eq!(1, offer.categories.len());
    }
}
