use errors::*;
use super::{Fold, Folder, FolderVar, Shifter};

/// Sometimes we wish to fold two values with a distinct deBruijn
/// depth (i.e., you want to fold `(A, B)` where A is defined under N
/// binders, and B under N+K).  In that case, you can fold
/// `(Shifted::new(K, A), B)`, and we will map a var of 0 in A to a
/// var of K. This only makes sense if the first N binders of both A
/// and B are the same binders, of course.
#[derive(Debug)]
pub struct Shifted<T: Fold> {
    adjustment: usize,
    value: T,
}

impl<T: Fold> Shifted<T> {
    pub fn new(adjustment: usize, value: T) -> Self {
        Shifted { adjustment, value }
    }
}

impl<T: Fold> Fold for Shifted<T> {
    type Result = T::Result;

    fn fold_with(&self, folder: &mut Folder, binders: usize) -> Result<Self::Result> {
        // I... think this is right if binders is not zero, but not sure,
        // and don't care to think about it.
        assert_eq!(binders, 0);

        // First up-shift any variables. i.e., if `self.value`
        // contains a free var with index 0, and `self.adjustment ==
        // 2`, we will translate it to a free var with index 2; then
        // we will fold *that* through `folder`.
        let mut new_folder = (Shifter::new(self.adjustment), folder);
        self.value.fold_with(&mut new_folder, binders)
    }
}



