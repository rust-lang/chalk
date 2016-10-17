use formula::Formula;

#[cfg(test)]
mod test;

#[derive(Clone, Debug)]
pub struct Substitution<C> {
    data: Arc<SubstitutionData<C>>
}

deref_to!(Substitution<C>.data => SubstitutionData<C>);

impl<C> Substitution<C> {
    pub fn new(data: SubstitutionData<C>) -> Self {
        Substitution { data: data }
    }

    pub fn empty() -> Substitution<C> {
        Self::new(SubstitutionData::empty())
    }
}

pub struct SubstitutionData<C> {
    parent: Option<Substitution<C>>,
    values: Vec<Formula<C>>,
}

impl<C> SubstitutionData<C> {
    pub fn new(parent: Option<Substitution<C>>, values: Vec<Formula<C>>) -> Self {
        SubstitutionData { parent: parent, values: values }
    }

    pub fn push(&mut self, value: Formula<C>) {
        self.values.push(value);
    }

    pub fn empty() -> Self {
        Self::new(None, vec![])
    }

    pub fn lookup(&self, depth: usize) -> Option<&Formula<C>> {
        let values_len = self.values.len();
        if depth >= values_len {
            if let Some(parent) = self.parent.as_ref() {
                parent.lookup(depth - values_len)
            } else {
                None
            }
        } else {
            Some(&self.values[depth])
        }
    }
}

pub trait Substitute<C> {
    fn substitute(self, substitution: &Substitution<C>) -> Self;
}

pub struct Substituted<C,V> {
    substitution: Substitution<C>,
    value: V,
}

