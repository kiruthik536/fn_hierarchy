use syn::{Ident, Item};

#[derive(Debug)]
pub struct UserDefined<'a>{
    pub fns:Vec<&'a Ident>,
}

impl<'a> UserDefined<'a>{
    pub fn new(ast:&'a syn::File)->Self{
        let mut fns:Vec<&Ident> = Vec::new();
        for item in &ast.items{
            if let Item::Fn(function) = item{
                fns.push(&function.sig.ident);
            }
        }

        Self{ fns }
    }
}