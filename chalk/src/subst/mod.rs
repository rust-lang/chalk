use std::fmt::{self, Debug};
use std::sync::Arc;
use std::ops::Index;

#[cfg(test)]
mod test;

pub struct Subst<L: Debug> {
    link: Arc<SubstLink<L>>,
}

struct SubstLink<L: Debug> {
    value: L,
    parent: Option<Subst<L>>,
}

impl<L: Debug> Subst<L> {
    pub fn new(parent: Option<&Subst<L>>, value: L) -> Subst<L> {
        let parent = parent.cloned();
        Subst {
            link: Arc::new(SubstLink {
                parent: parent,
                value: value,
            }),
        }
    }

    pub fn root(value: L) -> Subst<L> {
        Self::new(None, value)
    }

    pub fn push(&self, value: L) -> Subst<L> {
        Self::new(Some(self), value)
    }

    pub fn get(&self, n: usize) -> &L {
        let mut p = self;
        for _ in 0..n {
            if let Some(ref parent) = p.link.parent {
                p = parent;
            } else {
                panic!("lookup error: n0={:?} in {:?}", n, self)
            }
        }
        &p.link.value
    }

    pub fn iter<'iter>(&'iter self) -> SubstIter<'iter, L> {
        SubstIter { link: Some(&*self.link) }
    }
}

impl<L: Debug> Clone for Subst<L> {
    fn clone(&self) -> Self {
        Subst { link: self.link.clone() }
    }
}

pub struct SubstIter<'iter, L: 'iter + Debug> {
    link: Option<&'iter SubstLink<L>>,
}

impl<'iter, L: Debug> Iterator for SubstIter<'iter, L> {
    type Item = &'iter L;

    fn next(&mut self) -> Option<Self::Item> {
        self.link.map(|link| {
            self.link = link.parent.as_ref().map(|s| &*s.link);
            &link.value
        })
    }
}

impl<'iter, L: Debug> IntoIterator for &'iter Subst<L> {
    type Item = &'iter L;
    type IntoIter = SubstIter<'iter, L>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<L: fmt::Debug> fmt::Debug for Subst<L> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        fmt.debug_list().entries(self).finish()
    }
}

impl<L: Debug> Index<usize> for Subst<L> {
    type Output = L;
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}
