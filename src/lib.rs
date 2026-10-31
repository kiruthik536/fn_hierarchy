use syn::{ImplItem, Item};

#[derive(Debug)]
pub enum FunctionKind{
    Standalone,
    Associated,
    Method,
}

#[derive(Debug)]
pub struct about_functions{
    function_kind:FunctionKind,
    function_name:String,
}
impl about_functions{
    fn new(item:&Item)->Vec<Self>{
        let mut vec_of_fns:Vec<about_functions> = Vec::new();
        match item{
            Item::Fn(function)=>{
                vec_of_fns.push(Self { function_kind: FunctionKind::Standalone, function_name:function.sig.ident.to_string()  });
            },
            Item::Impl(function)=>{
                for item in &function.items{
                    //println!("{:#?}",function);
                    if let ImplItem::Fn(function) = item{
                        if function.sig.receiver().is_some(){
                            vec_of_fns.push(Self { function_kind: FunctionKind::Method, function_name:function.sig.ident.to_string()  });
                        }else {
                            vec_of_fns.push(Self { function_kind: FunctionKind::Associated, function_name:function.sig.ident.to_string()  });
                        }
                    }
                }
            }
            _=>{},
        }
        vec_of_fns
    }
}

#[derive(Debug)]
pub struct UserDefined{
    pub fns:Vec<FunctionKind>,
}

impl UserDefined{
    pub fn new(ast:&syn::File)->Self{
        let mut fns:Vec<FunctionKind> = Vec::new();
        // for item in &ast.items{
        //     let vec = about_functions::new(item);
        //     println!("{:#?}",vec);
        //     // match item{
        //     //     Item::Fn(function)=>{
        //     //         // fns.push(FunctionKind::Standalone(function.sig.ident.to_string()));
        //     //     },
        //     //     Item::Impl(function)=>{
        //     //         for item in &function.items{
        //     //             println!("{:#?}",function);
        //     //             if let ImplItem::Fn(function) = item{
        //     //                 //println!("{:#?}",item);
        //     //                 if function.sig.receiver().is_some(){
        //     //                     //
        //     //                 }
        //     //             }
        //     //         }
        //     //     }
        //     //     _=>{},
        //     // }
        //     //println!("{:#?}",item);
        // }
        
        let vec:Vec<about_functions> = ast.items.iter().flat_map(|item| about_functions::new(item)).collect();
        //println!("{:#?}",vec);
        let len = vec.len();
        //println!("{:?}",len);
        Self{ fns }
    }
}


