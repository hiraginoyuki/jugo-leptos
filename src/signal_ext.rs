use core::cell::Cell;
use core::ops::{Deref, DerefMut};
use leptos::{SignalUpdate, SignalUpdateUntracked};
use std::rc::Rc;

pub struct UpdateGuard<'a, T: ?Sized> {
    inner: &'a mut T,
    updated: Rc<Cell<bool>>,
}
impl<'a, T: ?Sized> UpdateGuard<'a, T> {
    pub fn new(inner: &'a mut T) -> (Self, Rc<Cell<bool>>) {
        let updated = Rc::new(Cell::new(false));
        let cloned = Rc::clone(&updated);

        (Self { inner, updated }, cloned)
    }
}
impl<'a, T: ?Sized> Deref for UpdateGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
impl<'a, T: ?Sized> DerefMut for UpdateGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.updated.set(true);
        self.inner
    }
}

pub trait SignalUpdateConditional<T>: SignalUpdate<Value = T> + SignalUpdateUntracked<T> {
    fn update_guarded<R>(&self, f: impl FnOnce(UpdateGuard<T>) -> R) -> Option<R> {
        if let Some((true, value)) = self.try_update_untracked(move |value| {
            let (guard, updated) = UpdateGuard::new(value);
            let value = f(guard);
            (updated.get(), value)
        }) {
            self.update(|_| {});
            Some(value)
        } else {
            None
        }
    }
    fn update_if_some<R>(&self, f: impl FnOnce(&mut Self::Value) -> Option<R>) -> Option<R> {
        match self.try_update_untracked(f) {
            Some(Some(value)) => {
                self.update(|_| {});
                Some(value)
            }
            _ => None,
        }
    }
    fn update_if(&self, f: impl FnOnce(&mut Self::Value) -> bool) -> bool {
        match self.try_update_untracked(f) {
            Some(true) => {
                self.update(|_| {});
                true
            }
            _ => false,
        }
    }
}
impl<T, S: SignalUpdate<Value = T> + SignalUpdateUntracked<T>> SignalUpdateConditional<T> for S {}
