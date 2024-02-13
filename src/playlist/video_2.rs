#[macro_export]
macro_rules! avec {
    ($($element:expr),*) =>{{
        let mut vs = Vec::with_capacity($crate::avec![@CC; $($element),*]);
        $(vs.push($element);)*
        vs
    }};
    ($($element:expr),+ $(,)?) =>{{
        let mut vs = Vec::new();
        $(vs.push($element);)+
        vs
    }};

    ($($element:expr,)*) =>{{
        $crate::avec![$($element),*]
    }};

    ($element:expr; $count:expr) => {{
        let cunt = $count;
        let el = $element.clone();
        let mut vs = Vec::new();
        vs.resize(cunt, el);
        vs
    }};

    (@CC; $($element:expr),*) => { <[()]>::len(&[$($crate::avec![@SUBC; $element]),*]) };

    (@SUBC; $_element:expr) => {()};
    
}

#[test]
fn empty_vec() {
    let x : Vec<u32> = avec![];
    assert!(x.is_empty());
}

#[test]
fn singe_test(){
    let x :Vec<u32> = avec![42; 3];
    assert!(!x.is_empty());
    assert_eq!(x.len(),3);
}

#[test]
fn multiple(){
    let x : Vec<u32> = avec![123,13,2425,4456];
    assert!(!x.is_empty());
    assert_eq!(x.len(), 4);
}

//TODO ,implement macros for hashmap