use crate::models::Cart;
use rand::distributions::Alphanumeric;
use rand::thread_rng;
use rand::Rng;
use std::collections::HashMap;

#[derive(Debug)]
pub struct UserSession<'a> {
    id: String,
    cart: Option<Cart<'a>>,
}

impl<'a> UserSession<'a> {
    fn random_key(len: usize) -> String {
        thread_rng().sample_iter(&Alphanumeric).take(len).collect()
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn set_cart(&mut self, cart: Cart<'a>) {
        self.cart = Some(cart);
    }
}

impl<'a> UserSession<'a> {
    pub fn new(cart: Cart<'a>) -> Self {
        UserSession {
            cart: Some(cart),
            id: UserSession::random_key(48),
        }
    }
}

#[derive(Debug)]
pub struct SessionRegistry<'a> {
    sessions: HashMap<String, UserSession<'a>>,
}

impl<'a> SessionRegistry<'a> {
    pub fn new() -> Self {
        SessionRegistry {
            sessions: HashMap::new(),
        }
    }

    pub fn insert_session(&mut self, session: UserSession<'a>) {
        self.sessions.insert(session.id().to_owned(), session);
    }

    pub fn delete_cart(&mut self, key: &str) {
        self.sessions.remove(key);
    }
}
