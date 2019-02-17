
#[derive(Clone, Debug)]
pub(super) struct SelectorFragment {
    pub(super) tag_name: String,
    pub(super) id: String,
    pub(super) classes: Vec<String>,
}

impl SelectorFragment {
    pub(super) fn new() -> Self {
        Self {
            tag_name: String::new(),
            id: String::new(),
            classes: vec![],
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct SelectorQuery<'a> {
    pub(super) tag_name: &'a str,
    pub(super) id: &'a str,
    pub(super) classes: Box<[&'a str]>,
}

impl<'a> SelectorQuery<'a> {
    pub(super) fn new(tag_name: &'a str, id: &'a str, classes: Box<[&'a str]>) -> Self {
        Self {
            tag_name,
            id,
            classes,
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct Selector {
    pub(super) fragments: Vec<SelectorFragment>
}

impl Selector {
    pub(super) fn new() -> Self {
        Self {
            fragments: vec![]
        }
    }
    pub(super) fn get_index_classes(&self) -> Vec<String> {
        let mut ret = vec![];
        for frag in self.fragments.iter() {
            let s = if frag.classes.len() > 0 { frag.classes[0].clone() } else { String::from("") };
            if !ret.contains(&s) {
                ret.push(s)
            }
        }
        ret
    }
    pub(super) fn match_query<'a>(&self, query: &'a SelectorQuery) -> bool {
        for frag in self.fragments.iter() {
            if frag.tag_name.len() > 0 && frag.tag_name != query.tag_name { continue };
            if frag.id.len() > 0 && frag.id != query.id { continue };
            let mut class_matches = true;
            for class_name in frag.classes.iter() {
                if !query.classes.contains(&class_name.as_str()) {
                    class_matches = false;
                    break;
                }
            }
            if !class_matches { continue };
            return true;
        }
        false
    }
}
