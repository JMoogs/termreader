use crate::StatefulList;

pub struct MenuOptions {
    pub local_options: StatefulList<String>,
    pub global_options: StatefulList<String>,
    pub local_history_options: StatefulList<String>,
    pub global_history_options: StatefulList<String>,
    pub category_list: StatefulList<String>,
    pub source_options: StatefulList<String>,
    pub source_book_options: StatefulList<String>,
    pub category_options: StatefulList<String>,
}

impl MenuOptions {
    pub fn new(categories: Vec<String>) -> Self {
        let local_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("Move to category..."),
            String::from("Rename"),
            String::from("Start from beginning"),
            String::from("Remove book from library"),
        ]);

        let global_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("View chapter list"),
            String::from("Move to category..."),
            String::from("Rename"),
            String::from("Start from beginning"),
            String::from("Remove book from library"),
        ]);

        let local_history_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("Remove from history"),
        ]);

        let global_history_options = StatefulList::from(vec![
            String::from("Continue reading"),
            String::from("View book"),
            String::from("Remove from history"),
        ]);

        let category_list = StatefulList::from(categories);

        let source_options =
            StatefulList::from(vec![String::from("Search"), String::from("View popular")]);

        let source_book_options = StatefulList::from(vec![
            String::from("Start from beginning"),
            String::from("Add book to library"),
            // String::from("View chapters"),
        ]);

        let category_options = StatefulList::from(vec![
            String::from("Create categories"),
            String::from("Re-order categories (Not yet implemented)"),
            String::from("Rename categories"),
            String::from("Delete categories"),
        ]);

        Self {
            local_options,
            global_options,
            local_history_options,
            global_history_options,
            category_list,
            source_options,
            source_book_options,
            category_options,
        }
    }
}
