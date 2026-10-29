use std::{error::Error};
use lets_do_it::UserDefined;



fn main()->Result<(),Box<dyn Error>>{
    let code = r#"
        use std::collections::HashMap;

struct User {
    name: String,
    age: u32,
}

impl User {
    // Associated function
    fn new(name: String, age: u32) -> Self {
        Self { name, age }
    }

    // Method
    fn get_name(&self) -> &str {
        &self.name
    }

    fn get_age(&self) -> u32 {
        self.age
    }

    fn have_birthday(&mut self) {
        self.age += 1;
    }

    fn is_adult(&self) -> bool {
        self.age >= 18
    }

    fn rename(&mut self, new_name: String) {
        self.name = new_name;
    }
}

// Simple function
fn greet() {
    println!("Hello, Rust!");
}

// Function with parameters
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

fn divide(a: f64, b: f64) -> Option<f64> {
    if b == 0.0 {
        None
    } else {
        Some(a / b)
    }
}

// Function with a reference parameter
fn calculate_length(text: &str) -> usize {
    text.len()
}

// Function returning a String
fn create_message(name: &str) -> String {
    format!("Welcome, {}!", name)
}

// Function with a vector
fn sum_numbers(numbers: &[i32]) -> i32 {
    numbers.iter().sum()
}

fn find_max(numbers: &[i32]) -> Option<i32> {
    numbers.iter().copied().max()
}

// Generic function
fn identity<T>(value: T) -> T {
    value
}

// Generic function with a trait bound
fn print_value<T: std::fmt::Debug>(value: T) {
    println!("{:?}", value);
}

// Function using a tuple
fn min_max(numbers: &[i32]) -> Option<(i32, i32)> {
    let min = numbers.iter().copied().min()?;
    let max = numbers.iter().copied().max()?;
    Some((min, max))
}

// Function with a HashMap
fn count_words(text: &str) -> HashMap<String, usize> {
    let mut counts = HashMap::new();

    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }

    counts
}

// Function returning Result
fn parse_number(text: &str) -> Result<i32, std::num::ParseIntError> {
    text.parse::<i32>()
}

// Recursive function
fn factorial(n: u64) -> u64 {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

// Function taking a closure
fn apply_operation<F>(a: i32, b: i32, operation: F) -> i32
where
    F: Fn(i32, i32) -> i32,
{
    operation(a, b)
}

// Function with a lifetime
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

// Function returning an iterator
fn even_numbers(numbers: &[i32]) -> impl Iterator<Item = i32> + '_ {
    numbers.iter().copied().filter(|n| n % 2 == 0)
}

// Function with multiple return paths
fn classify_number(n: i32) -> &'static str {
    if n < 0 {
        "negative"
    } else if n == 0 {
        "zero"
    } else {
        "positive"
    }
}

fn main() {
    greet();

    println!("{}", add(10, 5));
    println!("{}", subtract(10, 5));
    println!("{}", multiply(10, 5));
    println!("{:?}", divide(10.0, 2.0));

    let mut user = User::new(String::from("Alex"), 20);
    user.have_birthday();
    user.rename(String::from("Sam"));

    println!("{}", user.get_name());
    println!("{}", user.get_age());
    println!("{}", user.is_adult());

    println!("{}", calculate_length("hello"));
    println!("{}", create_message("Rustacean"));
    println!("{}", sum_numbers(&[1, 2, 3, 4, 5]));
    println!("{:?}", find_max(&[1, 8, 3, 6]));
    println!("{:?}", min_max(&[1, 8, 3, 6]));

    print_value(identity(42));
    println!("{:?}", count_words("rust is fun rust is fast"));
    println!("{:?}", parse_number("123"));
    println!("{}", factorial(5));

    println!("{}", apply_operation(4, 3, |a, b| a * b));
    println!("{}", longest("hello", "world!"));
    println!("{:?}", even_numbers(&[1, 2, 3, 4, 5, 6]).collect::<Vec<_>>());

    println!("{}", classify_number(-10));
}"#;

    let ast:syn::File = syn::parse_str(code)?;
    //println!("{:#?}",ast);
    

    let fns = UserDefined::new(&ast).fns;
    println!("{:?}",fns);

    Ok(())
}
