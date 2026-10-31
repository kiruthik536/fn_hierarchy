use std::{error::Error};
use lets_do_it::UserDefined;



fn main()->Result<(),Box<dyn Error>>{
    let code = r#"

struct hello{
    name:String,
}

impl hello{
    fn hi(self){
        println!("kjdsfb");
    }

    fn h(){
        println!("df;lhgjl;");
    }
}

fn main(){
    
}


        "#;

    let ast:syn::File = syn::parse_str(code)?;
    

    let fns = UserDefined::new(&ast).fns;
    //println!("{:?}",fns);

    Ok(())
}
