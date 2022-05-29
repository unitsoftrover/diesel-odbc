mod a1;

trait T1{
    type Ty;
    fn test(&self){
        println!("T1");
    }
}

#[derive(Default, Debug)]
struct TA{
    id : i64,
    name : String
}

#[derive(Default, Debug)]
struct TB{
    id : i64,
    name : String,
    class : String,
}

impl T1 for TA{
    type Ty = Self;
    fn test(&self){
        println!("TA id:{}, name:{}", self.id, self.name);
        
    }
}

impl T1 for TB{
    type Ty = Self;
    fn test(&self){
        println!("TB id:{}, name:{} class:{}", self.id, self.name, self.class);
    }
}

fn call<T:T1>(t1 : &dyn T1<Ty=T>)
{
    t1.test();
}


fn main(){
    let ta = TA{
        id:1,
        name: "rover".to_string(),      
    };
    call(&ta);
    let tb = TB{
        id:1,
        name: "rover".to_string(),
        class : Default::default(),
    };
    call(&tb);

    println!("A");
    a1::a1::test_a1();
}

