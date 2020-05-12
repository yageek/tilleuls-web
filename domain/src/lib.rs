use chrono::{Date, Utc};

#[derive(Debug)]
pub struct Item {
    title: String,
    unit: String,
}

impl Item {
    pub fn new<T: Into<String>, U: Into<String>>(title: T, unit: U) -> Item {
        Item {
            title: title.into(),
            unit: unit.into(),
        }
    }
}

#[derive(Debug)]
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

pub struct WeeklyBasketOffer {
    #[allow(dead_code)]
    date: Date<Utc>,
    categories: Vec<Category>,
}

impl WeeklyBasketOffer {
    pub fn new(date: Date<Utc>, categories: Vec<Category>) -> WeeklyBasketOffer {
        WeeklyBasketOffer { date, categories }
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

        let item = Item::new("Fraise", "250 gr");
        fruits.add_item(item);

        assert_eq!(1, fruits.items.len());

        let offer = WeeklyBasketOffer::new(Utc::today(), vec![fruits]);
        assert_eq!(1, offer.categories.len());
    }
}
