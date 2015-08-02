thread_local! {
    THE_ARENA: RefCell<Arena> = RefCell::new(Arena::new())
}

pub fn write<FUNC>(func: FUNC)
    where FUNC: FnOnce(&mut Arena)
{
    THE_ARENA.with(|t| write(t.borrow_mut()))
}

pub fn read<FUNC>(func: FUNC)
    where FUNC: FnOnce(&mut Arena)
{
    THE_ARENA.with(|t| read(t.borrow()))
}
