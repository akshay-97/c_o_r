use::std::cell::UnsafeCell;
use std::ops::Index;
use::std::ops::{Deref, DerefMut};
use std::pin::Pin;
use::std::ptr::NonNull;
use::std::marker::PhantomData;
use std::fmt::Display;


struct Cell<T>
{
    value: UnsafeCell<T>
}

impl<T : Copy> Cell<T>{
    fn new(value:T) -> Self{
        Cell{value: UnsafeCell::new(value)}
    }
    
    fn get(&self) -> T {
        unsafe {
            *self.value.get()
        }
    }

    fn set(&self,value : T){
        unsafe {
            *self.value.get() = value;
        }
    }
}

// TODO: why do we need to own data in RefCell implementation

// SAFETY: UnsafeCell doesnt implement sync => RefCell and Cell dont implement sync
#[derive(Copy, Clone)]
enum RefState{
    Open,
    Voyeur(usize),
    Access
}

//RefCell Implementation
struct RefCellInner <T>{
    value : UnsafeCell<T>,
    reference: Cell<RefState>,
}

impl <T> RefCellInner<T> {
    fn new(value:T) -> Self{
        Self { value : UnsafeCell::new(value), reference : Cell::new(RefState::Open)}
    }

    fn borrow(&self) -> Option<Ref<'_,T>> {
        match self.reference.get(){
            RefState::Open => unsafe { 
                self.reference.set(RefState::Voyeur(1));
                Some(Ref{inner : self})
            },
            RefState::Voyeur(n) => unsafe { 
                self.reference.set(RefState::Voyeur(n+1));
                Some(Ref{inner: self})
            },
            RefState::Access => None
        }
    }

    fn borrow_mut(&self) -> Option<RefMut<'_, T>>{
        match self.reference.get(){
            RefState::Access | RefState::Voyeur(_) => None,
            RefState::Open => {
                self.reference.set(RefState::Access);
                Some(RefMut{inner:self})
            }
        }
    }
}


struct Ref<'rc, T>{
    inner : &'rc RefCellInner<T>
}

impl <T> Drop for Ref<'_, T>{
    fn drop(&mut self) {
        unsafe{
            match (*self.inner).reference.get(){
                RefState::Access => panic!("unreachable case"),
                RefState::Voyeur(0) => panic!("unreachable case"),
                RefState::Voyeur(n) => {
                    //(*self.inner).ref_count.set(n-1);
                    if n ==1 {
                        (*self.inner).reference.set(RefState::Open);
                    }
                    else{
                        (*self.inner).reference.set(RefState::Voyeur(n-1));
                    }
                },
                RefState::Open => panic!("unreachable case")
            }
        }
    }
}

struct RefMut<'rc, T>{
    inner: &'rc RefCellInner<T>
}

impl <T> Drop for RefMut<'_,T>{
    fn drop(&mut self){
        match (*self.inner).reference.get(){
            RefState::Access => (*self.inner).reference.set(RefState::Open),
            _ => panic!("unreachable case")
        }
    }
}

impl <T> Deref for Ref<'_, T>{
    type Target = T;
    fn deref(&self) -> &Self::Target{
        //SAFETY: reference only given if shared ref or no ref exists
        unsafe {
            &*self.inner.value.get()
        }
    }
}

impl <T> Deref for RefMut<'_, T>{
    type Target = T;
    fn deref(&self) -> &Self::Target{
        //SAFETY: same as deref for ref
        unsafe{ &*self.inner.value.get()}
    }
}
impl <T> DerefMut for RefMut<'_, T>{
    //type Target = T;
    fn deref_mut(&mut self) -> &mut Self::Target{
        //SAFETY: Exclusive reference to inner here, not possible by others
        unsafe{
            &mut *self.inner.value.get()
        }
    } 
}

//Rc implementation
// Not send or sync
struct RcInner<T> {
    value : T,
    ref_count : Cell<isize>,
}

struct Rc<T>{
    inner: NonNull<RcInner<T>>,
    _marker : PhantomData<RcInner<T>>
}

impl <T> Rc<T>{
    fn new(value :T) -> Self{
        let inner = Box::new(RcInner { value, ref_count : Cell::new(0)});
        Self {
            inner: unsafe{ NonNull::new_unchecked(Box::into_raw(inner))}, _marker : PhantomData
        }
    }
}

impl <T> Clone for Rc<T> {
    fn clone(&self) -> Self{
            let inner = unsafe { self.inner.as_ref()};
            inner.ref_count.set(inner.ref_count.get() +1 );
            Self { inner : self.inner, _marker : PhantomData}
    }
}

impl <T> Deref for Rc<T>{
    type Target = T;
    fn deref(&self) -> &Self::Target{
        &unsafe { self.inner.as_ref()}.value
    }
}

impl <T> Drop for Rc<T>{
    fn drop(&mut self) {
            let inner = unsafe { self.inner.as_ref()} ;
            match inner.ref_count.get(){
                0 => panic!("unreachable code"),
                1=> {
                    drop(inner); // proactively dropping inner so that it doesnt get referenced  in the following lines (upto the end of function)
                    let _ = unsafe {Box::from_raw(self.inner.as_ptr())};
                },
                n => {
                    inner.ref_count.set(n-1);
                } 
            }
    }
}

// struct Inspector<'a, 'b, T, U: Display>(&'a u8, &'b u8, T, U);

// unsafe impl<#[may_dangle]'a, #[may_dangle] 'b, T, U: Display> Drop for Inspector<'a, 'b, T, U> {
//     fn drop(&mut self) {
//         println!("Inspector({})", self.3);
//     }
// }

// struct Outer<'a ,'b, T, U:Display>{
//     inspect : Option<Inspector<'a, 'b, T,U>>,
//     refer1 : Box<u8>,
// }

struct Foo<'a, T : Default>{
    v : &'a mut T
}

impl <T:Default> Drop for Foo<'_, T>{
    fn drop(&mut self){
        std::mem::replace(self.v, T::default());
    }
}
fn main(){
    let mut t;
    let foo;
    t = String::from("aasdf");
    foo = Rc::new(Foo { v : &mut t});
    // let mut outer = Outer {inspect: None, refer1 : Box::new(0)};
    // outer.inspect = Some(Inspector(&outer.refer1, &outer.refer1, String::from("asdf"), "random"));
    let (y,x): (Rc<&String>, String);
    x = String::new();
    let y = Rc::new(&x);

}