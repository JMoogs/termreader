use crate::{book::BookRef, books_context::BooksContext, id::ID, TRError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(super) struct LibCtxSerialize {
    pub(super) books: HashMap<String, Vec<ID>>,
    pub(super) default_category_name: String,
    pub(super) category_order: Vec<String>,
}

impl LibCtxSerialize {
    pub(super) fn from_lib_ctx(lib_ctx: LibraryContext) -> Self {
        let books = lib_ctx
            .books
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|b| b.get_id()).collect()))
            .collect();

        Self {
            books,
            default_category_name: lib_ctx.default_category_name,
            category_order: lib_ctx.category_order,
        }
    }

    pub(super) fn to_lib_ctx(self, books: &BooksContext) -> LibraryContext {
        let books = self
            .books
            .into_iter()
            .map(|(k, v)| (k, v.into_iter().map(|id| books.get(id).unwrap()).collect()))
            .collect();

        LibraryContext {
            books,
            default_category_name: self.default_category_name,
            category_order: self.category_order,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct LibraryContext {
    pub(super) books: HashMap<String, Vec<BookRef>>,
    pub(super) default_category_name: String,
    pub(super) category_order: Vec<String>,
}

impl LibraryContext {
    /// Creates an empty library
    pub(super) fn new() -> Self {
        let mut map = HashMap::new();
        map.insert(String::from("Default"), Vec::new());
        Self {
            books: map,
            default_category_name: String::from("Default"),
            category_order: vec![String::from("Default")],
        }
    }

    pub(super) fn remove_book(&mut self, id: ID) {
        let lists = self.books.values_mut();

        for list in lists {
            let search = list.iter().position(|i| i.get_id() == id);
            if search.is_none() {
                continue;
            }
            let pos = search.unwrap();
            list.remove(pos);
        }
    }

    // /// Add a book to a given category, or the default category if a category isn't given
    // ///
    // /// This function checks if the book is already in the user's library before adding it
    // pub(super) fn add_book(&mut self, book: Book, category: Option<&str>) {
    //     todo!()
    // }

    // pub(super) fn move_category(&mut self, id: ID, category_name: Option<&str>) {
    //     // Don't move to a non-exsiting category
    //     if category_name.is_some_and(|cat| !self.get_categories().contains(&cat.to_string())) {
    //         return;
    //     }
    //     if let Some(book) = self.find_book(id).cloned() {
    //         // If the category is the same as the current one do nothing
    //         if let (Some(b_cat), Some(cat)) = (&book.category, category_name) {
    //             if b_cat == cat {
    //                 return;
    //             }
    //         }

    //         self.remove_book(id);
    //         self.add_book(book, category_name);
    //     }
    // }

    pub(super) fn create_category(&mut self, name: String) -> Result<(), TRError> {
        // Don't allow multiple categories with the same name.
        if self.books.contains_key(&name) {
            return Err(TRError::Duplicate);
        }
        self.books.insert(name.clone(), Vec::new());
        self.category_order.push(name);
        Ok(())
    }

    pub(super) fn delete_category(&mut self, name: String) -> Result<(), TRError> {
        // Don't allow the default category to be deleted.
        if self.default_category_name == name {
            return Err(TRError::InvalidChoice(String::from(
                "the default category cannot be deleted",
            )));
        }

        if let Some(mut v) = self.books.remove(&name) {
            let new_l = self.books.get_mut(&self.default_category_name).unwrap();
            v.iter_mut().for_each(|b| b.0.borrow_mut().category = None);
            new_l.append(&mut v);
            self.category_order.retain(|x| x != &name)
        }

        Ok(())
    }

    pub(super) fn rename_category(
        &mut self,
        old_name: String,
        new_name: String,
    ) -> Result<(), TRError> {
        // Don't allow multiple categories with the same name.
        if self.books.contains_key(&new_name) {
            return Err(TRError::Duplicate);
        }

        if let Some(v) = self.books.remove(&old_name) {
            if self.default_category_name == old_name {
                self.default_category_name = new_name.clone();
            }

            for val in self.category_order.iter_mut() {
                if val == &old_name {
                    let _ = std::mem::replace(val, new_name.clone());
                }
            }
            self.books.insert(new_name, v);
        }

        Ok(())
    }

    pub(super) fn get_categories(&self) -> &Vec<String> {
        &self.category_order
    }

    pub(super) fn get_category_count(&self) -> usize {
        self.books.len()
    }

    pub(super) fn reorder_category_forwards(&mut self, move_idx: usize) -> Option<usize> {
        let cat_count = self.category_order.len();
        if move_idx >= cat_count {
            // Index out of bounds
            return None;
        } else if move_idx == 0 {
            // First category so can't move up
            return Some(0);
        } else {
            // Valid to swap upwards
            self.category_order.swap(move_idx, move_idx - 1);
            return Some(move_idx - 1);
        }
    }

    pub(super) fn reorder_category_backwards(&mut self, move_idx: usize) -> Option<usize> {
        let cat_count = self.category_order.len();
        if move_idx >= cat_count {
            // Index out of bounds
            return None;
        } else if move_idx == cat_count - 1 {
            // Last category so can't move down
            return Some(move_idx);
        } else {
            // Valid to swap downwards
            self.category_order.swap(move_idx, move_idx + 1);
            return Some(move_idx + 1);
        }
    }
}
