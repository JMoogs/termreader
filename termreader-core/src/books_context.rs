use std::rc::Rc;
use std::{cell::RefCell, collections::HashMap};

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::{
    book::{Book, BookRef},
    id::ID,
};

#[serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(super) struct BooksContext {
    #[serde_as(as = "Vec<(_, _)>")]
    pub(super) books: HashMap<ID, BookRef>,
}

impl BooksContext {
    pub(super) fn new() -> Self {
        Self {
            books: HashMap::new(),
        }
    }

    pub(super) fn add_book(&mut self, book: Book) {
        if self.books.get(&book.get_id()).is_none() {
            self.books
                .insert(book.get_id(), BookRef(Rc::new(RefCell::new(book))));
        }
    }

    pub(super) fn get(&self, id: ID) -> Option<BookRef> {
        let b = self.books.get(&id);
        match b {
            None => return None,
            Some(book) => Some(BookRef::clone(book)),
        }
    }

    pub(super) fn find_book(&self, id: ID) -> Option<BookRef> {
        self.get(id)
    }

    pub(super) fn find_book_by_url(&self, url: String) -> Option<BookRef> {
        self.books
            .values()
            .find(|x| {
                x.0.borrow()
                    .get_full_url()
                    .is_some_and(|link| link.to_string() == url)
            })
            .cloned()
    }

    /// Removes any books that are no longer referenced, returning the number of books removed
    pub(super) fn remove_unneeded(&mut self) -> usize {
        let mut to_remove = Vec::new();
        for book in self.books.values() {
            // Strong count of 1 implies that the books are only stored in this map
            // Strong count of the inner type should always be the same as the amount of `BookRef`s
            if Rc::strong_count(&book.0) == 1 {
                to_remove.push(book.0.borrow().get_id());
            }
        }

        let rem_count = to_remove.len();
        for id in to_remove {
            self.books.remove(&id);
        }

        return rem_count;
    }
}
