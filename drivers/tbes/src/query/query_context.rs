use core::{fmt::Display, iter::*};

use alloc::vec;
use alloc::{string::String, vec::Vec};

pub struct QueryContext {
    log_query_plan: bool,
    buffer: String,
    open_sections: Vec<&'static str>,
}

impl QueryContext {
    pub fn new(log_query_plan: bool) -> Self {
        Self {
            log_query_plan,
            buffer: String::new(),
            open_sections: vec![],
        }
    }

    pub fn open_section(&mut self, name: &'static str) {
        if !self.log_query_plan {
            return;
        }
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<').chain(name.chars()).chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
        self.open_sections.push(name);
    }

    pub fn close_section(&mut self) {
        if !self.log_query_plan {
            return;
        }
        let name = self
            .open_sections
            .pop()
            .expect("You should call 'close_section' only if you have actually opened a section!");
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<')
            .chain(once('/'))
            .chain(name.chars())
            .chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
    }

    pub fn item(&mut self, name: &str) {
        if !self.log_query_plan {
            return;
        }
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<')
            .chain(name.chars())
            .chain(once('/'))
            .chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
    }

    pub fn item_vec<'a>(&mut self, args: impl IntoIterator<Item = &'a str>) {
        if !self.log_query_plan {
            return;
        }
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<')
            .chain(args.into_iter().flat_map(str::chars))
            .chain(once('/'))
            .chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
    }
}

impl Display for QueryContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.buffer)
    }
}
