/* Open questions:
1. why doesnt our flatten require Sized constraint
2. fn vs fnMut, why std::flat_map defines as fnMut
3. external trait, see https://www.youtube.com/watch?v=yozQ9C69pNs&list=PLqbS7AVVErFiWDOAVrPt7aYmnuuOLYvOa&index=3
*/
pub struct Flatten<O>
where
    O : Iterator,
    O::Item : IntoIterator
{
    outer :O,
    current_front_inner: Option<<O::Item as IntoIterator>::IntoIter>,
    current_rear_inner: Option<<O::Item as IntoIterator>::IntoIter>
}

impl<O> Flatten<O>
where
    O : Iterator,
    O::Item : IntoIterator
{
    fn new(iter:O) -> Self{
        Flatten {outer : iter, current_front_inner : None, current_rear_inner : None}
    }
}

pub fn flatten<O>(iter:O) -> Flatten<O>
where
    O : Iterator,
    O::Item : IntoIterator
{
    Flatten::new(iter)
}

impl <O> Iterator for Flatten<O>
where
    O : Iterator,
    <O as Iterator>::Item: IntoIterator,
{
    type Item = <O::Item as IntoIterator>::Item;
    fn next(&mut self) -> Option<Self::Item>{
        loop {
            if let Some(ref mut f) = self.current_front_inner {
                if let Some(i) = f.next() {
                    return Some(i)
                }
                self.current_front_inner = None;
            }
            
            let outer_next = match self.outer.next() {
                None => {
                    return self.current_rear_inner.as_mut()?.next()
                },
                Some(val) => Some(val.into_iter())
                
            };
            self.current_front_inner = outer_next;
        }
    }
}


impl <O> DoubleEndedIterator for Flatten<O> 
where
    O: DoubleEndedIterator,
    O::Item: IntoIterator,
    <O::Item as IntoIterator> ::IntoIter : DoubleEndedIterator
{ 
    fn next_back(&mut self) -> Option<Self::Item>{
        loop {
            if let Some(ref mut f) = self.current_rear_inner {
                if let Some(i) = f.next_back(){
                    return Some(i)
                }
                self.current_rear_inner = None;
            }

            //let inner_iter = self.outer.next_back()?.into_iter();
            let last_inner = match self.outer.next_back() {
                None => {
                    return self.current_front_inner.as_mut()?.next_back()
                },
                Some(val) => Some(val.into_iter())
            };
            self.current_rear_inner = last_inner;
        }
    }
} 
#[cfg(test)]
mod tests{
    use super::*;

    // #[test]
    // fn one(){
    //     assert_eq!(flatten(std::iter::once(vec![0])).count(), 1)
    // }

    // #[test]
    // fn void(){
    //     assert_eq!(flatten(vec![vec![1], vec![1,2,3], vec![3,4]].into_iter()).count(), 6)
    // }
    #[test]
    fn two(){
        assert_eq!(flatten(std::iter::once(vec!["a", "b"])).count(), 2)
    }

    #[test]
    fn reverse(){
        let mut iter = flatten(vec![vec!["a", "b"], vec!["c", "d"]].into_iter());
        assert_eq!(Some("a"),iter.next());
        assert_eq!(Some("d"), iter.next_back());
        assert_eq!(Some("b"),iter.next());
        assert_eq!(Some("c"), iter.next_back());
        assert_eq!(None,iter.next());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn reverse_front_asymm(){
        let mut iter = flatten(vec![vec!["a"], vec!["b", "c"]].into_iter());
        assert_eq!(Some("a"), iter.next());
        assert_eq!(Some("c"), iter.next_back());
        assert_eq!(Some("b"), iter.next());
    }

    #[test]
    fn reverse_end_asymm(){
        let mut iter = flatten(vec![vec!["a", "b"], vec!["c"]].into_iter());
        assert_eq!(Some("a"), iter.next());
        assert_eq!(Some("c"), iter.next_back());
        assert_eq!(Some("b"), iter.next_back());
    }

}
/*
TODO: why do you want to use fnMut here?? 
*/

struct FlatMap_Ours<O, F ,U>
where
O : Iterator,
U : IntoIterator,
F: Fn(O::Item) -> U
{
    structure : O,
    mapper : F,
    inner_s : Option<<U as IntoIterator>::IntoIter>
}

impl <O, U, F> FlatMap_Ours<O, F, U>
where
O : Iterator,
U : IntoIterator,
F : Fn(O::Item) -> U
{
    fn new_t(iter : O, mapper : F) -> FlatMap_Ours<O, F, U>{
        FlatMap_Ours { structure : iter, mapper, inner_s : None}
    }
}

fn flat_map_ours <O, F, U> (iter: O ,mapper_fn : F) -> FlatMap_Ours<O, F, U>
where
    O : Iterator,
    U : IntoIterator,
    F: Fn(O::Item) -> U
{
    FlatMap_Ours::new_t(iter, mapper_fn)
}

impl <O, U, F> Iterator for FlatMap_Ours<O, F, U>
where
    O : Iterator,
    U: IntoIterator,
    F : Fn(<O as Iterator>::Item) -> U
{
    type Item = <U as IntoIterator>::Item;
    fn next(&mut self) -> Option<Self::Item>{
        loop {
            if let Some(inner) = &mut self.inner_s{
                if let Some(inner_value) = inner.next(){
                    return Some(inner_value)
                }
                self.inner_s = None;
            }
            let outer_iter_value = self.structure.next()?;
            self.inner_s = Some((self.mapper)(outer_iter_value).into_iter());
        }
    }
}
