use core::iter::*;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

#[derive(Default)]
pub struct QueryWriter {
    buffer: String,
    open_sections: Vec<&'static str>,
}

impl QueryWriter {
    pub fn open_section(&mut self, name: &'static str) {
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<').chain(name.chars()).chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
        self.open_sections.push(name);
    }

    pub fn close_section(&mut self) {
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
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<')
            .chain(name.chars())
            .chain(once('/'))
            .chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
    }

    pub fn item_vec<'a>(&mut self, args: impl IntoIterator<Item = &'a str>) {
        let spaces = repeat_n(' ', self.open_sections.len());
        let tag = once('<')
            .chain(args.into_iter().flat_map(str::chars))
            .chain(once('/'))
            .chain(once('>'));
        self.buffer.extend(spaces.chain(tag).chain(once('\n')));
    }
}

impl ToString for QueryWriter {
    fn to_string(&self) -> String {
        self.buffer.clone()
    }
}
