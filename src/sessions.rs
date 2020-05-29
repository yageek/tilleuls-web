use crate::models::Cart;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug)]
pub struct UserSession {
    id: String,
    cart: Option<Cart>,
}

impl UserSession {
    fn random_key(len: usize) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(len).collect()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_cart(&mut self, cart: Cart) {
        self.cart = Some(cart);
    }
}

impl UserSession {
    pub fn new(cart: Cart) -> Self {
        UserSession {
            cart: Some(cart),
            id: UserSession::random_key(48),
        }
    }
}

#[derive(Debug)]
pub struct SessionRegistry {
    sessions: HashMap<String, UserSession>,
}

impl<'a> SessionRegistry {
    pub fn new() -> Self {
        SessionRegistry {
            sessions: HashMap::new(),
        }
    }

    pub fn insert_session(&mut self, session: UserSession) {
        self.sessions.insert(session.id().to_owned(), session);
    }

    pub fn delete_cart(&mut self, key: &str) {
        self.sessions.remove(key);
    }
}
