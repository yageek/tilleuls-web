use chrono::{Date, Utc};
use serde::Serialize;

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
